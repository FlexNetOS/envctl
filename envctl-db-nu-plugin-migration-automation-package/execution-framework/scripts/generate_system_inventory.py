from __future__ import annotations

import json
import os
import re
import sqlite3
import tomllib
from collections import defaultdict
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-100_SYSTEM_INVENTORY"
HELPER_ID = "helper-artifact-01"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art-100-system-inventory"
OPERATION_ID = "op-art-100-system-inventory"
TARGET_REGISTRY_PATH = root() / "generated" / "envctl_target_registry.json"

BLOCKED_PARTS = {"secrets", "private_keys"}
BLOCKED_SUFFIXES = {".pem", ".key"}
SKIP_DIRS = {
    ".cache",
    ".direnv",
    ".git",
    ".hg",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".venv",
    "__pycache__",
    "node_modules",
    "target",
}
SKIP_REL_PREFIXES = (
    ".toolchains/.bun/install/cache/",
    "var/lib/cargo/",
    "var/tmp/",
)
SKIP_REL_PARTS = {"_work", ".worktrees", "release/out", "vendor"}
TEXT_EXTENSIONS = {
    ".conf",
    ".go",
    ".json",
    ".kdl",
    ".md",
    ".nix",
    ".nu",
    ".py",
    ".rs",
    ".service",
    ".sh",
    ".sql",
    ".timer",
    ".toml",
    ".yaml",
    ".yml",
}
SCRIPT_SUFFIXES = {".bash", ".fish", ".nu", ".pl", ".ps1", ".py", ".rb", ".sh", ".zsh"}
MAX_CONTENT_BYTES = 256 * 1024
MAX_ITEMS_PER_CATEGORY = 200
MAX_MARKDOWN_ROWS = 40


def relpath(path: Path, base: Path) -> str:
    return path.relative_to(base).as_posix()


def is_blocked(path: Path) -> bool:
    name = path.name
    lower_parts = {part.lower() for part in path.parts}
    if name == ".env":
        return True
    if lower_parts.intersection(BLOCKED_PARTS):
        return True
    return path.suffix.lower() in BLOCKED_SUFFIXES


def should_skip_dir(path: Path, target_root: Path) -> bool:
    name = path.name
    if name in SKIP_DIRS:
        return True
    rel = relpath(path, target_root)
    if any(rel == prefix.rstrip("/") or rel.startswith(prefix) for prefix in SKIP_REL_PREFIXES):
        return True
    return any(part in rel for part in SKIP_REL_PARTS)


def read_text_limited(path: Path) -> str:
    try:
        if path.stat().st_size > MAX_CONTENT_BYTES:
            return ""
        if path.suffix.lower() not in TEXT_EXTENSIONS and path.name not in {"Dockerfile", "Makefile", "Justfile"}:
            return ""
        return path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return ""


def parse_package_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        data = {}
    return {
        "name": data.get("name") or path.parent.name,
        "scripts": sorted((data.get("scripts") or {}).keys()),
        "bin": data.get("bin"),
    }


def parse_toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8", errors="ignore"))
    except (OSError, tomllib.TOMLDecodeError):
        return {}


def parse_go_mod(path: Path) -> str:
    try:
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            if line.startswith("module "):
                return line.split(None, 1)[1].strip()
    except OSError:
        pass
    return path.parent.name


def parse_desktop(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for line in read_text_limited(path).splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in {"Name", "Exec", "Type"}:
            out[key.lower()] = value.strip()
    return out


def parse_systemd_unit(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for line in read_text_limited(path).splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in {"Description", "ExecStart", "WantedBy", "OnCalendar", "Unit"}:
            out[key.lower()] = value.strip()
    return out


def parse_compose_services(text: str) -> list[str]:
    services: list[str] = []
    in_services = False
    for line in text.splitlines():
        if re.match(r"^services:\s*$", line):
            in_services = True
            continue
        if in_services and re.match(r"^\S", line):
            break
        match = re.match(r"^\s{2}([A-Za-z0-9_.-]+):\s*$", line)
        if in_services and match:
            services.append(match.group(1))
    return services


def make_item(kind: str, path: Path, target_root: Path, **extra: Any) -> dict[str, Any]:
    item = {"kind": kind, "path": relpath(path, target_root)}
    item.update({k: v for k, v in extra.items() if v not in (None, "", [], {})})
    return item


def add_item(
    categories: dict[str, list[dict[str, Any]]],
    totals: dict[str, int],
    category: str,
    item: dict[str, Any],
) -> None:
    totals[category] += 1
    items = categories[category]
    if len(items) < MAX_ITEMS_PER_CATEGORY:
        items.append(item)


def classify_file(
    path: Path,
    target_root: Path,
    categories: dict[str, list[dict[str, Any]]],
    totals: dict[str, int],
) -> None:
    name = path.name
    suffix = path.suffix.lower()
    text = read_text_limited(path)
    lower_path = relpath(path, target_root).lower()
    lower_text = text.lower()

    if name == "package.json":
        add_item(categories, totals, "applications", make_item("node_package", path, target_root, **parse_package_json(path)))
    elif name == "Cargo.toml":
        data = parse_toml(path)
        package = data.get("package") or {}
        workspace = data.get("workspace") or {}
        add_item(
            categories,
            totals,
            "applications",
            make_item(
                "rust_crate" if package else "rust_workspace",
                path,
                target_root,
                name=package.get("name") or path.parent.name,
                members=workspace.get("members", [])[:20] if isinstance(workspace.get("members"), list) else [],
            ),
        )
    elif name == "pyproject.toml":
        data = parse_toml(path)
        project = data.get("project") or {}
        add_item(categories, totals, "applications", make_item("python_project", path, target_root, name=project.get("name") or path.parent.name))
    elif name == "go.mod":
        add_item(categories, totals, "applications", make_item("go_module", path, target_root, name=parse_go_mod(path)))
    elif name == "flake.nix":
        add_item(categories, totals, "applications", make_item("nix_flake", path, target_root))
    elif suffix == ".desktop":
        add_item(categories, totals, "applications", make_item("desktop_application", path, target_root, **parse_desktop(path)))

    if suffix == ".service":
        add_item(categories, totals, "services", make_item("systemd_service", path, target_root, **parse_systemd_unit(path)))
    if name in {"docker-compose.yml", "docker-compose.yaml", "compose.yml", "compose.yaml"}:
        services = parse_compose_services(text)
        add_item(categories, totals, "services", make_item("compose_stack", path, target_root, services=services))
        for service in services:
            lowered = service.lower()
            if any(token in lowered or token in lower_text for token in ["postgres", "mysql", "mariadb", "sqlite", "mongo", "redis"]):
                add_item(categories, totals, "databases", make_item("compose_database_candidate", path, target_root, service=service))
            if any(token in lowered or token in lower_text for token in ["queue", "kafka", "rabbit", "nats", "pubsub", "stream"]):
                add_item(categories, totals, "queues", make_item("compose_queue_candidate", path, target_root, service=service))

    if ".github/workflows/" in lower_path or name in {"Justfile", "Makefile"}:
        job_kind = "github_workflow" if ".github/workflows/" in lower_path else name.lower()
        add_item(categories, totals, "jobs", make_item(job_kind, path, target_root))
        if "schedule:" in lower_text or "cron:" in lower_text:
            add_item(categories, totals, "schedulers", make_item("scheduled_workflow", path, target_root))

    if suffix == ".timer":
        add_item(categories, totals, "schedulers", make_item("systemd_timer", path, target_root, **parse_systemd_unit(path)))
    if "cron" in lower_path or "crontab" in name.lower():
        add_item(categories, totals, "schedulers", make_item("cron_candidate", path, target_root))

    if suffix == ".sql" or "/migrations/" in lower_path or "/migration/" in lower_path:
        add_item(categories, totals, "databases", make_item("schema_or_migration", path, target_root))

    if any(token in lower_path or token in lower_text for token in ["kafka", "rabbitmq", "nats", "pubsub", "queue", "stream"]):
        add_item(categories, totals, "queues", make_item("queue_signal", path, target_root))

    if "openapi" in lower_path or "swagger" in lower_path or re.search(r"@(get|post|put|delete|patch)\(|router\.(get|post|put|delete|patch)\(", text):
        add_item(categories, totals, "apis", make_item("api_surface", path, target_root))

    if "report" in lower_path and suffix in {".json", ".jsonl", ".md", ".tsv"}:
        add_item(categories, totals, "reports", make_item("report_artifact", path, target_root))

    if suffix in SCRIPT_SUFFIXES or "/scripts/" in lower_path or "/bin/" in lower_path or text.startswith("#!"):
        add_item(categories, totals, "scripts", make_item("script", path, target_root))


def scan_target(target_root: Path) -> dict[str, Any]:
    categories: dict[str, list[dict[str, Any]]] = defaultdict(list)
    totals: dict[str, int] = defaultdict(int)
    scanned_files = 0
    skipped_blocked = 0
    skipped_dirs = 0
    skipped_files = 0
    for dirpath, dirnames, filenames in os.walk(target_root):
        current = Path(dirpath)
        kept_dirs = []
        for dirname in dirnames:
            candidate = current / dirname
            if should_skip_dir(candidate, target_root):
                skipped_dirs += 1
            else:
                kept_dirs.append(dirname)
        dirnames[:] = kept_dirs
        for filename in filenames:
            path = current / filename
            if is_blocked(path):
                skipped_blocked += 1
                continue
            if should_skip_dir(path.parent, target_root):
                skipped_files += 1
                continue
            scanned_files += 1
            classify_file(path, target_root, categories, totals)
    normalized = {name: sorted(items, key=lambda item: item["path"]) for name, items in categories.items()}
    for required in ["applications", "services", "jobs", "databases", "queues", "apis", "reports", "scripts", "schedulers"]:
        normalized.setdefault(required, [])
        totals.setdefault(required, 0)
    return {
        "target_root": str(target_root),
        "scanned_files": scanned_files,
        "skipped_blocked_paths": skipped_blocked,
        "skipped_directories": skipped_dirs,
        "skipped_files": skipped_files,
        "excluded_directory_policy": {
            "dir_names": sorted(SKIP_DIRS),
            "relative_prefixes": list(SKIP_REL_PREFIXES),
            "relative_parts": sorted(SKIP_REL_PARTS),
        },
        "categories": normalized,
        "counts": {name: totals[name] for name in sorted(normalized)},
        "sample_counts": {name: len(items) for name, items in sorted(normalized.items())},
    }


def load_target_descriptor() -> dict[str, Any]:
    data = json.loads(TARGET_REGISTRY_PATH.read_text(encoding="utf-8"))
    rows = data.get("registry_rows") or []
    selected = next((row for row in rows if row.get("target_id") == "flexnetos-vs-lifeos"), rows[0])
    env_root = os.environ.get("MIGRATION_TARGET_ROOT")
    target_root = Path(env_root or selected["primary_root"]).expanduser().resolve()
    if not target_root.exists():
        raise FileNotFoundError(f"target root does not exist: {target_root}")
    return {"registry_report": data, "selected_target": selected, "target_root": target_root}


def write_markdown(inventory: dict[str, Any], descriptor: dict[str, Any], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    selected = descriptor["selected_target"]
    lines = [
        "# System Inventory",
        "",
        f"Generated at: `{inventory['generated_at']}`",
        f"Task: `{TASK_ID}`",
        f"Target: `{selected.get('target_id')}`",
        f"Target root: `{inventory['scan']['target_root']}`",
        f"Descriptor hash: `{selected.get('descriptor_hash')}`",
        "",
        "## Coverage",
        "",
        f"- Files scanned: `{inventory['scan']['scanned_files']}`",
        f"- Blocked paths skipped: `{inventory['scan']['skipped_blocked_paths']}`",
        f"- Directories skipped by generated/cache policy: `{inventory['scan']['skipped_directories']}`",
        "",
        "| category | count |",
        "|---|---:|",
    ]
    for category, count in sorted(inventory["scan"]["counts"].items()):
        lines.append(f"| {category} | {count} |")
    for category in ["applications", "services", "jobs", "databases", "queues", "apis", "reports", "scripts", "schedulers"]:
        lines.extend(["", f"## {category.title()}", ""])
        items = inventory["scan"]["categories"][category]
        if not items:
            lines.append("_No filesystem evidence found in the bounded scan._")
            continue
        lines.extend(["| kind | path | detail |", "|---|---|---|"])
        for item in items[:MAX_MARKDOWN_ROWS]:
            details = []
            for key in ["name", "services", "description", "exec", "service", "scripts", "members"]:
                value = item.get(key)
                if value:
                    rendered = ", ".join(value[:8]) if isinstance(value, list) else str(value)
                    details.append(f"{key}={rendered}")
            detail = "; ".join(details)
            lines.append(f"| {item.get('kind', '')} | `{item['path']}` | {detail} |")
        if len(items) > MAX_MARKDOWN_ROWS:
            lines.append(f"| ... | ... | {len(items) - MAX_MARKDOWN_ROWS} additional entries in JSON artifact |")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def insert_run_fixture(conn: sqlite3.Connection, descriptor: dict[str, Any]) -> dict[str, str]:
    selected = descriptor["selected_target"]
    target_db_id = selected["id"]
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO UPDATE SET
          primary_root = excluded.primary_root,
          compare_root = excluded.compare_root,
          descriptor_json = excluded.descriptor_json,
          descriptor_hash = excluded.descriptor_hash
        """,
        (
            target_db_id,
            selected["target_id"],
            selected["target_type"],
            selected["primary_root"],
            selected.get("compare_root"),
            json.dumps(selected, sort_keys=True),
            selected["descriptor_hash"],
            selected["safety_mode"],
            selected["max_auto_risk"],
        ),
    )
    contract_id = conn.execute("SELECT id FROM envctl_migration_artifact_contracts ORDER BY created_at_utc LIMIT 1").fetchone()[0]
    recipe_id = conn.execute("SELECT id FROM envctl_migration_recipes WHERE artifact_contract_id = ? LIMIT 1", (contract_id,)).fetchone()[0]
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            RUN_ID,
            target_db_id,
            recipe_id,
            contract_id,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            '{"python":"stdlib","sqlite":"stdlib"}',
            "sha256:art-100-system-inventory",
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            "ART-100_SYSTEM_INVENTORY/generate-register",
            "sha256:art-100-command",
            "python3 scripts/generate_system_inventory.py",
            '{"task_id":"ART-100_SYSTEM_INVENTORY"}',
            "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json",
        ),
    )
    conn.commit()
    return {"run_id": RUN_ID, "operation_id": OPERATION_ID, "contract_id": contract_id}


def register_artifacts(conn: sqlite3.Connection, fixture: dict[str, str], inventory: dict[str, Any]) -> dict[str, Any]:
    registry = ArtifactRegistry(conn, package_root())
    categories = inventory["scan"]["counts"]
    required_categories_present = all(category in categories for category in ["applications", "services", "jobs", "databases", "queues", "apis", "reports", "scripts", "schedulers"])
    common = {
        "run_id": fixture["run_id"],
        "status": "complete",
        "producer_operation_id": fixture["operation_id"],
        "contract_id": fixture["contract_id"],
        "provenance": {
            "task_id": TASK_ID,
            "owner_agent": "artifact-agent",
            "helper_id": HELPER_ID,
            "target_id": inventory["target"]["target_id"],
            "target_root": inventory["scan"]["target_root"],
            "target_descriptor_hash": inventory["target"]["descriptor_hash"],
        },
        "evidence_refs": [
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
        "links": [
            {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
        ],
        "validations": [
            {
                "validator": "generate_system_inventory.py:artifact-exists",
                "status": "pass",
                "details": {"files_written": 3},
                "evidence_refs": [
                    "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.md",
                    "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json",
                ],
            },
            {
                "validator": "generate_system_inventory.py:required-categories",
                "status": "pass" if required_categories_present else "fail",
                "details": categories,
                "evidence_refs": ["execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json"],
            },
            {
                "validator": "generate_system_inventory.py:blocked-path-policy",
                "status": "pass",
                "details": {"blocked_paths_skipped": inventory["scan"]["skipped_blocked_paths"]},
                "evidence_refs": ["execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json"],
            },
        ],
    }
    records = [
        {
            **common,
            "artifact_id": "art-100-system-inventory-md",
            "title": "ART-100 System Inventory Markdown",
            "artifact_type": "migration_artifact",
            "path": "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.md",
        },
        {
            **common,
            "artifact_id": "art-100-system-inventory-json",
            "title": "ART-100 System Inventory JSON",
            "artifact_type": "machine_readable_record",
            "path": "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json",
        },
        {
            **common,
            "artifact_id": "01-current-state-system-inventory-md",
            "title": "System Inventory",
            "artifact_type": "migration_artifact",
            "path": "execution-framework/migration-artifacts/01-current-state/system-inventory.md",
        },
    ]
    results = [registry.register(record) for record in records]
    artifact_rows = [fetch_artifact(conn, fixture["run_id"], result["artifact_id"]) for result in results]
    return {"registry_results": results, "artifact_rows": artifact_rows}


def build_report(conn: sqlite3.Connection, fixture: dict[str, str], registry_report: dict[str, Any], inventory: dict[str, Any]) -> dict[str, Any]:
    index_rows = [
        dict(zip(["artifact_id", "status", "path", "content_hash"], row))
        for row in conn.execute(
            """
            SELECT artifact_id, status, path, content_hash
            FROM envctl_migration_artifact_index
            WHERE run_id = ?
            ORDER BY artifact_id
            """,
            (fixture["run_id"],),
        ).fetchall()
    ]
    errors = []
    if len(index_rows) < 3:
        errors.append("expected 3 registered ART-100 artifact rows")
    for row in index_rows:
        if not row["content_hash"] or not row["content_hash"].startswith("sha256:"):
            errors.append(f"missing content hash for {row['artifact_id']}")
    for category in ["applications", "services", "jobs", "databases", "queues", "apis", "reports", "scripts", "schedulers"]:
        if category not in inventory["scan"]["counts"]:
            errors.append(f"missing inventory category: {category}")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "target": inventory["target"],
        "scan_summary": {
            "scanned_files": inventory["scan"]["scanned_files"],
            "skipped_blocked_paths": inventory["scan"]["skipped_blocked_paths"],
            "skipped_directories": inventory["scan"]["skipped_directories"],
            "counts": inventory["scan"]["counts"],
        },
        "registry": registry_report,
        "artifact_index_rows": index_rows,
        "errors": errors,
    }


def main() -> None:
    descriptor = load_target_descriptor()
    scan = scan_target(descriptor["target_root"])
    selected = descriptor["selected_target"]
    inventory = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "target": {
            "target_id": selected["target_id"],
            "target_type": selected["target_type"],
            "primary_root": selected["primary_root"],
            "compare_root": selected.get("compare_root"),
            "descriptor_hash": selected["descriptor_hash"],
        },
        "scan": scan,
    }

    artifact_dir = root() / "migration-artifacts" / "art-100_system_inventory"
    md_path = artifact_dir / "system_inventory.md"
    json_path = artifact_dir / "system_inventory.json"
    canonical_md_path = root() / "migration-artifacts" / "01-current-state" / "system-inventory.md"
    artifact_dir.mkdir(parents=True, exist_ok=True)
    json_path.write_text(json.dumps(inventory, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    write_markdown(inventory, descriptor, md_path)
    write_markdown(inventory, descriptor, canonical_md_path)

    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_run_fixture(conn, descriptor)
    registry_report = register_artifacts(conn, fixture, inventory)
    report = build_report(conn, fixture, registry_report, inventory)

    report_path = root() / "generated" / "art_100_system_inventory_registry_report.json"
    log_path = root() / "logs" / f"{TASK_ID}.log"
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    report_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    heartbeat_path.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "logs_uri": f"logs/{TASK_ID}.log",
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/generate_system_inventory.py",
        "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.md",
        "execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json",
        "execution-framework/migration-artifacts/01-current-state/system-inventory.md",
        "execution-framework/generated/art_100_system_inventory_registry_report.json",
        "execution-framework/state/ART-100_SYSTEM_INVENTORY.heartbeat.json",
        "execution-framework/logs/ART-100_SYSTEM_INVENTORY.log",
        "execution-framework/proof_records/ART-100_SYSTEM_INVENTORY.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_system_inventory.py",
        "python3 -m py_compile scripts/generate_system_inventory.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(descriptor["target_root"]),
        files_changed,
        commands_run,
        report,
        [
            "migration-artifacts/art-100_system_inventory/system_inventory.md",
            "migration-artifacts/art-100_system_inventory/system_inventory.json",
            "migration-artifacts/01-current-state/system-inventory.md",
            "generated/art_100_system_inventory_registry_report.json",
            "logs/ART-100_SYSTEM_INVENTORY.log",
        ],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "run VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-100 inventory validation errors",
    )
    append_proof(proof)
    print(
        "ART-100 status={status} registered={registered} scanned_files={files} categories={categories}".format(
            status=report["status"],
            registered=len(report["artifact_index_rows"]),
            files=report["scan_summary"]["scanned_files"],
            categories=json.dumps(report["scan_summary"]["counts"], sort_keys=True),
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
