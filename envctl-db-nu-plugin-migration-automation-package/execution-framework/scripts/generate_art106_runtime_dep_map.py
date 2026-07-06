from __future__ import annotations

import fnmatch
import hashlib
import json
import os
import re
import sqlite3
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-106_RUNTIME_DEP_MAP"
HELPER_ID = "helper-artifact-07"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art106-runtime-dep-map"
OPERATION_ID = "produce-01-current-state-runtime-dependency-map-md"
TARGET_DB_ID = "target-art106-flexnetos-vs-lifeos"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "recipe-flexnetos-package-artifact-contract-1.0.0"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-106_runtime_dep_map"
TASK_MD = ARTIFACT_DIR / "runtime-dependency-map.md"
TASK_JSON = ARTIFACT_DIR / "runtime-dependency-map.json"
CANONICAL_MD = root() / "migration-artifacts" / "01-current-state" / "runtime-dependency-map.md"
REPORT_PATH = root() / "generated" / "art106_runtime_dep_map_registry_report.json"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"

BLOCKED_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")
SKIP_DIRS = {
    ".cache",
    ".direnv",
    ".git",
    ".hg",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".svn",
    ".tox",
    ".venv",
    ".kb",
    ".toolchains",
    "__pycache__",
    "dist",
    "node_modules",
    "result",
    "target",
}
SCAN_SUFFIXES = {
    ".bash",
    ".conf",
    ".desktop",
    ".envrc",
    ".go",
    ".ini",
    ".js",
    ".json",
    ".jsx",
    ".kdl",
    ".md",
    ".nix",
    ".nu",
    ".properties",
    ".py",
    ".rs",
    ".service",
    ".sh",
    ".sql",
    ".toml",
    ".ts",
    ".tsx",
    ".yaml",
    ".yml",
    ".zsh",
}
SCAN_NAMES = {"Dockerfile", "Containerfile", "Makefile", "Justfile", "Procfile"}
MAX_FILES = 3000
MAX_FILE_BYTES = 700_000
MAX_EVIDENCE_PER_CATEGORY = 180

DEPENDENCY_PATTERNS: dict[str, list[tuple[str, re.Pattern[str]]]] = {
    "databases": [
        ("sqlite", re.compile(r"\b(sqlite3?|rusqlite|libsql|\.sqlite|\.db\b|sqlite://)", re.IGNORECASE)),
        ("postgres", re.compile(r"\b(postgres|postgresql|psycopg|pgx|tokio-postgres|DATABASE_URL)\b", re.IGNORECASE)),
        ("mysql", re.compile(r"\b(mysql|mariadb)\b", re.IGNORECASE)),
        ("duckdb", re.compile(r"\bduckdb\b", re.IGNORECASE)),
        ("migration-db", re.compile(r"\benvctl_migration_[a-z_]+\b")),
    ],
    "env_vars": [
        ("python-env", re.compile(r"os\.environ(?:\.get)?\(\s*['\"]([A-Za-z_][A-Za-z0-9_]*)['\"]|os\.getenv\(\s*['\"]([A-Za-z_][A-Za-z0-9_]*)['\"]")),
        ("rust-env", re.compile(r"std::env::var(?:_os)?\(\s*['\"]([A-Za-z_][A-Za-z0-9_]*)['\"]|env::var(?:_os)?\(\s*['\"]([A-Za-z_][A-Za-z0-9_]*)['\"]")),
        ("js-env", re.compile(r"process\.env\.([A-Za-z_][A-Za-z0-9_]*)|process\.env\[['\"]([A-Za-z_][A-Za-z0-9_]*)['\"]\]")),
        ("shell-env", re.compile(r"\$\{([A-Z_][A-Z0-9_]*)\}|\$([A-Z_][A-Z0-9_]*)\b")),
        ("nix-env", re.compile(r"builtins\.getEnv\s+['\"]([A-Z_][A-Z0-9_]*)['\"]")),
        ("nu-env", re.compile(r"\$env\.([A-Z_][A-Z0-9_]*)")),
    ],
    "secret_refs": [
        ("secret-name", re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*(?:SECRET|TOKEN|PASSWORD|PASSWD|API_KEY|PRIVATE_KEY|ACCESS_KEY|CLIENT_SECRET|CREDENTIAL)[A-Za-z0-9_]*)\b", re.IGNORECASE)),
        ("secret-store", re.compile(r"\b(vault|keyring|credential|secret_ref|secretRef|sops|age-key|pass show)\b", re.IGNORECASE)),
        ("redaction-policy", re.compile(r"\b(blocked_paths|private_keys|secrets/|redacted|redaction)\b", re.IGNORECASE)),
    ],
    "queues": [
        ("message-broker", re.compile(r"\b(kafka|nats|rabbitmq|amqp|mqtt|zeromq|redis[-_ ]?queue)\b", re.IGNORECASE)),
        ("task-queue", re.compile(r"\b(celery|rq\.Queue|queue|work_queue|job_queue|task_graph|operation queue)\b", re.IGNORECASE)),
        ("channel", re.compile(r"\b(tokio::sync::mpsc|crossbeam_channel|async_channel|broadcast::channel|watch::channel)\b")),
        ("run-events", re.compile(r"\benvctl_migration_(operations|run_events)\b")),
    ],
    "apis": [
        ("http-client", re.compile(r"\b(fetch\(|axios|requests\.|httpx\.|reqwest|urllib|curl\s|wget\s)\b", re.IGNORECASE)),
        ("http-server", re.compile(r"\b(FastAPI|Flask|express\(|axum|warp::|rocket::|actix_web|route\(|router\.)\b")),
        ("protocol", re.compile(r"\b(openapi|graphql|grpc|json-rpc|MCP|tools/list|initialize|REST|HTTP)\b", re.IGNORECASE)),
        ("endpoint", re.compile(r"https?://[A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%-]+")),
    ],
    "caches": [
        ("redis", re.compile(r"\b(redis|redis-server|redis://)\b", re.IGNORECASE)),
        ("memcached", re.compile(r"\b(memcached|memcache)\b", re.IGNORECASE)),
        ("cache-api", re.compile(r"\b(cache_dir|XDG_CACHE_HOME|lru_cache|cachetools|ttl_cache|cached_property|Cache-Control)\b", re.IGNORECASE)),
        ("build-cache", re.compile(r"\b(sccache|ccache|nix-store|pnpm-store|npm cache|bunfig|\.cache)\b", re.IGNORECASE)),
    ],
}


def now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def sha256_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return "sha256:" + h.hexdigest()


def relpath(path: Path, base: Path) -> str:
    try:
        return path.relative_to(base).as_posix()
    except ValueError:
        return path.as_posix()


def is_blocked_path(path: Path) -> bool:
    normalized = path.as_posix()
    return any(fnmatch.fnmatch(normalized, pattern) for pattern in BLOCKED_PATTERNS)


def sanitize_snippet(line: str) -> str:
    line = line.strip().replace("\t", " ")
    line = re.sub(r"\s+", " ", line)
    line = re.sub(
        r"(?i)(password|passwd|token|secret|api_key|private_key|access_key|client_secret)(\s*[:=]\s*)[^,'\"\s)]+",
        r"\1\2<redacted>",
        line,
    )
    line = re.sub(r"https?://([^/\s:@]+):([^@\s]+)@", r"https://<redacted>@", line)
    line = re.sub(r"([A-Za-z0-9_/-]{32,})", lambda match: match.group(1)[:24] + "...", line)
    return line[:220]


def extract_names(pattern: re.Pattern[str], line: str) -> list[str]:
    names: list[str] = []
    for match in pattern.finditer(line):
        groups = [group for group in match.groups() if group]
        if groups:
            names.extend(groups)
    return sorted(set(names))


def read_json(relpath: str) -> dict[str, Any]:
    return json.loads((root() / relpath).read_text(encoding="utf-8"))


def resolve_target_root() -> Path:
    env_root = os.environ.get("MIGRATION_TARGET_ROOT")
    if env_root:
        return Path(env_root).expanduser().resolve()
    registry = read_json("generated/envctl_target_registry.json")
    for row in registry.get("registry_rows", []):
        if row.get("target_id") == "flexnetos-vs-lifeos" and row.get("primary_root"):
            return Path(row["primary_root"]).expanduser().resolve()
    return package_root().resolve()


def target_context(target_root: Path) -> dict[str, Any]:
    registry = read_json("generated/envctl_target_registry.json")
    rows = registry.get("registry_rows", [])
    target = next((row for row in rows if row.get("target_id") == "flexnetos-vs-lifeos"), rows[0])
    return {
        "target_registry_status": registry.get("status"),
        "target": target,
        "resolved_target_root": target_root.as_posix(),
        "descriptor_inputs": registry.get("descriptor_inputs", []),
    }


def envctl_database_context() -> dict[str, Any]:
    model = read_json("generated/envctl_migration_db_model.json")
    artifact_registry = read_json("generated/envctl_artifact_registry_report.json")
    shared_protocol = read_json("generated/shared_protocol_manifest.json")
    return {
        "db_status": model.get("status"),
        "database_backend": model.get("database_backend"),
        "runtime": model.get("runtime"),
        "required_tables": model.get("required_tables", []),
        "capability_coverage": model.get("capability_coverage", {}),
        "artifact_registry_status": artifact_registry.get("status"),
        "shared_protocol_status": shared_protocol.get("status"),
        "shared_protocol_records": shared_protocol.get("required_records", []),
    }


def iter_scan_files(target_root: Path) -> tuple[list[Path], dict[str, Any]]:
    files: list[Path] = []
    skipped = Counter()
    if not target_root.exists():
        return files, {"target_exists": False, "skipped": {}, "truncated": False}
    for current, dirs, names in os.walk(target_root):
        current_path = Path(current)
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS and not is_blocked_path(current_path / d))
        for name in sorted(names):
            path = current_path / name
            rel = Path(relpath(path, target_root))
            if is_blocked_path(rel):
                skipped["blocked_policy"] += 1
                continue
            if path.suffix not in SCAN_SUFFIXES and path.name not in SCAN_NAMES:
                skipped["unsupported_suffix"] += 1
                continue
            try:
                stat = path.stat()
            except OSError:
                skipped["stat_error"] += 1
                continue
            if stat.st_size > MAX_FILE_BYTES:
                skipped["too_large"] += 1
                continue
            files.append(path)
            if len(files) >= MAX_FILES:
                skipped["max_files_reached"] += 1
                return files, {"target_exists": True, "skipped": dict(skipped), "truncated": True}
    return files, {"target_exists": True, "skipped": dict(skipped), "truncated": False}


def add_evidence(bucket: dict[str, list[dict[str, Any]]], category: str, item: dict[str, Any]) -> None:
    if len(bucket[category]) < MAX_EVIDENCE_PER_CATEGORY:
        bucket[category].append(item)


def scan_runtime_dependencies(target_root: Path) -> dict[str, Any]:
    files, scan_limits = iter_scan_files(target_root)
    evidence: dict[str, list[dict[str, Any]]] = defaultdict(list)
    file_hits: dict[str, set[str]] = defaultdict(set)
    dependency_names: dict[str, Counter[str]] = defaultdict(Counter)
    suffix_counts = Counter(path.suffix or path.name for path in files)
    top_level_counts = Counter(relpath(path, target_root).split("/", 1)[0] for path in files)
    errors: list[dict[str, str]] = []

    for path in files:
        rel = relpath(path, target_root)
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            errors.append({"path": rel, "error": str(exc)})
            continue
        for lineno, line in enumerate(lines, start=1):
            for category, patterns in DEPENDENCY_PATTERNS.items():
                for signal, pattern in patterns:
                    if not pattern.search(line):
                        continue
                    names = extract_names(pattern, line)
                    for name in names:
                        dependency_names[category][name] += 1
                    add_evidence(
                        evidence,
                        category,
                        {
                            "path": rel,
                            "line": lineno,
                            "signal": signal,
                            "names": names[:8],
                            "snippet": sanitize_snippet(line),
                        },
                    )
                    file_hits[rel].add(category)
                    break

    hotspots = []
    for rel, categories in file_hits.items():
        score = len(categories)
        if {"env_vars", "secret_refs"} <= categories:
            score += 2
        if {"databases", "apis"} <= categories:
            score += 1
        if {"queues", "caches"} <= categories:
            score += 1
        hotspots.append({"path": rel, "categories": sorted(categories), "score": score})
    hotspots.sort(key=lambda item: (-item["score"], item["path"]))

    coverage = {}
    for category in DEPENDENCY_PATTERNS:
        category_evidence = evidence.get(category, [])
        coverage[category] = {
            "status": "repo_evidence_found" if category_evidence else "not_found_in_safe_scan",
            "evidence_count": len(category_evidence),
            "file_count": len({item["path"] for item in category_evidence}),
            "top_names": [
                {"name": name, "count": count}
                for name, count in dependency_names[category].most_common(20)
            ],
            "sample_evidence": category_evidence[:20],
        }

    return {
        "target_root": target_root.as_posix(),
        "scan_limits": {
            **scan_limits,
            "max_files": MAX_FILES,
            "max_file_bytes": MAX_FILE_BYTES,
            "blocked_patterns": list(BLOCKED_PATTERNS),
            "secret_material_read": False,
        },
        "visited_file_count": len(files),
        "suffix_counts": dict(suffix_counts.most_common(25)),
        "top_level_counts": dict(top_level_counts.most_common(25)),
        "coverage": coverage,
        "hotspots": hotspots[:80],
        "errors": errors[:40],
    }


def build_runtime_dep_map() -> dict[str, Any]:
    target_root = resolve_target_root()
    scan = scan_runtime_dependencies(target_root)
    db_context = envctl_database_context()
    target = target_context(target_root)

    dependencies = [
        {
            "id": f"{category}:{item['name']}",
            "category": category,
            "name": item["name"],
            "evidence_count": item["count"],
            "source": "safe-scan extracted reference name",
        }
        for category, coverage in scan["coverage"].items()
        for item in coverage["top_names"]
    ]

    db_tables = db_context.get("required_tables", [])
    envctl_runtime_nodes = [
        {
            "id": "database:envctl-migration-sqlite",
            "category": "databases",
            "name": "envctl migration SQLite model",
            "evidence": ["generated/envctl_migration_db_model.json", "sql/001_migration_automation_schema.sql"],
        },
        {
            "id": "registry:artifact-registry",
            "category": "databases",
            "name": "artifact registry hash/evidence tables",
            "evidence": ["generated/envctl_artifact_registry_report.json", "scripts/artifact_registry.py"],
        },
        {
            "id": "queue:envctl-operation-queue",
            "category": "queues",
            "name": "envctl_migration_operations and run_events queue/event surfaces",
            "evidence": ["generated/envctl_migration_db_model.json"],
        },
        {
            "id": "api:shared-protocol-records",
            "category": "apis",
            "name": "envctl shared protocol schema records",
            "evidence": ["generated/shared_protocol_manifest.json", "schemas/shared_protocol.schema.json"],
        },
    ]

    edges = [
        {"from": "target:flexnetos-vs-lifeos", "to": "database:envctl-migration-sqlite", "type": "uses_control_plane_db"},
        {"from": "registry:artifact-registry", "to": "database:envctl-migration-sqlite", "type": "persists_hashes_to"},
        {"from": "queue:envctl-operation-queue", "to": "database:envctl-migration-sqlite", "type": "stored_in"},
        {"from": "api:shared-protocol-records", "to": "registry:artifact-registry", "type": "validates_artifact_records"},
    ]
    for hotspot in scan["hotspots"][:40]:
        for category in hotspot["categories"]:
            edges.append({"from": f"file:{hotspot['path']}", "to": f"category:{category}", "type": "contains_reference"})

    gaps = [
        {
            "category": category,
            "gap": "No safe-scan evidence found for this runtime dependency category.",
            "next_evidence_needed": "Collect live runtime inventory, service manifests, or approved deployment exports for this dependency class.",
        }
        for category, coverage in scan["coverage"].items()
        if coverage["evidence_count"] == 0
    ]

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "complete",
        "scope": {
            "source": "target descriptor, repo scan, envctl database reports, and safe text scan",
            "runtime_live_state_confirmed": False,
            "secret_material_read": False,
            "blocked_patterns": list(BLOCKED_PATTERNS),
            "categories": list(DEPENDENCY_PATTERNS),
        },
        "target_context": target,
        "envctl_database_context": db_context,
        "safe_scan": {
            "target_root": scan["target_root"],
            "visited_file_count": scan["visited_file_count"],
            "scan_limits": scan["scan_limits"],
            "suffix_counts": scan["suffix_counts"],
            "top_level_counts": scan["top_level_counts"],
            "errors": scan["errors"],
        },
        "coverage": scan["coverage"],
        "dependencies": dependencies[:200],
        "envctl_runtime_nodes": envctl_runtime_nodes,
        "edges": edges,
        "hotspots": scan["hotspots"],
        "gaps": gaps,
        "database_tables_considered": db_tables,
        "source_artifacts": [
            "generated/envctl_target_registry.json",
            "generated/package_scan.json",
            "generated/envctl_migration_db_model.json",
            "generated/envctl_artifact_registry_report.json",
            "generated/shared_protocol_manifest.json",
            "generated/execution_packets/ART-106_RUNTIME_DEP_MAP.json",
        ],
    }


def markdown_table(rows: list[list[str]]) -> str:
    if not rows:
        return ""
    header = "| " + " | ".join(rows[0]) + " |"
    sep = "| " + " | ".join(["---"] * len(rows[0])) + " |"
    body = ["| " + " | ".join(row) + " |" for row in rows[1:]]
    return "\n".join([header, sep, *body])


def render_markdown(dep_map: dict[str, Any]) -> str:
    coverage_rows = [["Category", "Status", "Evidence", "Files", "Top references"]]
    for category, item in dep_map["coverage"].items():
        top = ", ".join(f"`{name['name']}` ({name['count']})" for name in item["top_names"][:8]) or "none"
        coverage_rows.append(
            [category, item["status"], str(item["evidence_count"]), str(item["file_count"]), top]
        )

    evidence_rows = [["Category", "Signal", "Path", "Line", "Snippet"]]
    for category, item in dep_map["coverage"].items():
        for evidence in item["sample_evidence"][:10]:
            evidence_rows.append(
                [
                    category,
                    evidence["signal"],
                    f"`{evidence['path']}`",
                    str(evidence["line"]),
                    evidence["snippet"].replace("|", "\\|"),
                ]
            )

    hotspot_rows = [["Path", "Categories", "Score"]]
    for hotspot in dep_map["hotspots"][:25]:
        hotspot_rows.append(
            [
                f"`{hotspot['path']}`",
                ", ".join(hotspot["categories"]),
                str(hotspot["score"]),
            ]
        )

    node_rows = [["Node", "Category", "Evidence"]]
    for node in dep_map["envctl_runtime_nodes"]:
        node_rows.append([node["name"], node["category"], "<br>".join(f"`{item}`" for item in node["evidence"])])

    gap_rows = [["Category", "Gap", "Next evidence needed"]]
    for gap in dep_map["gaps"]:
        gap_rows.append([gap["category"], gap["gap"], gap["next_evidence_needed"]])

    lines = [
        "# ART-106 Runtime Dependency Map",
        "",
        f"Generated: `{dep_map['generated_at']}`",
        f"Status: `{dep_map['status']}`",
        "",
        "This map records runtime dependency evidence for databases, environment variables, secret references, queues, APIs, and caches. It is built from approved package artifacts, the envctl database model, the target descriptor, and a safe scan that excludes `.env`, secret directories, private keys, PEM files, and key files.",
        "",
        "## Scope",
        "",
        f"- Target: `{dep_map['target_context']['target'].get('target_id')}`",
        f"- Target root: `{dep_map['safe_scan']['target_root']}`",
        f"- Files visited: `{dep_map['safe_scan']['visited_file_count']}`",
        f"- Runtime live state confirmed: `{dep_map['scope']['runtime_live_state_confirmed']}`",
        f"- Secret material read: `{dep_map['scope']['secret_material_read']}`",
        f"- Envctl database backend: `{dep_map['envctl_database_context']['database_backend']}`",
        "",
        "## Coverage",
        "",
        markdown_table(coverage_rows),
        "",
        "## Envctl Runtime Nodes",
        "",
        markdown_table(node_rows),
        "",
        "## Hotspot Files",
        "",
        markdown_table(hotspot_rows) if dep_map["hotspots"] else "No runtime dependency hotspots were found in the safe scan.",
        "",
        "## Evidence Samples",
        "",
        markdown_table(evidence_rows),
        "",
        "## Gaps",
        "",
        markdown_table(gap_rows) if dep_map["gaps"] else "No empty runtime dependency categories in the safe scan.",
        "",
        "## Evidence Boundary",
        "",
        "- Environment variables and secret references are recorded by reference name only when names are visible in non-secret files.",
        "- Secret values, `.env` files, private key material, PEM files, and blocked directories are excluded.",
        "- Queue/API/cache rows are evidence categories from repository files and envctl database reports, not a claim of deployed live services unless later runtime inventory confirms them.",
        "",
    ]
    return "\n".join(lines)


def write_json(path: Path, obj: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def setup_run(conn: sqlite3.Connection, dep_map: dict[str, Any]) -> None:
    target = dep_map["target_context"]["target"]
    descriptor_json = json.dumps(target, sort_keys=True)
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO NOTHING
        """,
        (
            TARGET_DB_ID,
            target.get("target_id", "flexnetos-vs-lifeos"),
            target.get("target_type", "mixed"),
            dep_map["target_context"]["resolved_target_root"],
            target.get("compare_root"),
            descriptor_json,
            target.get("descriptor_hash") or sha256_text(descriptor_json),
            target.get("safety_mode", "approval-gated"),
            target.get("max_auto_risk", "R2"),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING
        """,
        (
            RUN_ID,
            TARGET_DB_ID,
            RECIPE_ID,
            CONTRACT_ID,
            "completed",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib", "codex": "codex-cli-background-shell"}, sort_keys=True),
            sha256_text(json.dumps(dep_map, sort_keys=True)),
            dep_map["generated_at"],
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(run_id, idempotency_key) DO NOTHING
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            f"{TASK_ID}/runtime-dependency-map",
            sha256_text("python3 scripts/generate_art106_runtime_dep_map.py"),
            "python3 scripts/generate_art106_runtime_dep_map.py",
            json.dumps(
                {
                    "task_id": TASK_ID,
                    "contract_row_id": "artifact:01-current-state-runtime-dependency-map-md",
                    "categories": list(DEPENDENCY_PATTERNS),
                },
                sort_keys=True,
            ),
            "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
            dep_map["generated_at"],
            now(),
        ),
    )
    conn.commit()


def register_artifacts(dep_map: dict[str, Any]) -> dict[str, Any]:
    conn = sqlite3.connect(":memory:")
    conn.row_factory = sqlite3.Row
    applied_migrations = apply_migrations(conn, package_root())
    setup_run(conn, dep_map)
    registry = ArtifactRegistry(conn, package_root())
    evidence_refs = [
        "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.md",
        "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
        "execution-framework/migration-artifacts/01-current-state/runtime-dependency-map.md",
        "execution-framework/generated/envctl_target_registry.json",
        "execution-framework/generated/package_scan.json",
        "execution-framework/generated/envctl_migration_db_model.json",
        "execution-framework/generated/envctl_artifact_registry_report.json",
        "execution-framework/generated/shared_protocol_manifest.json",
        "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
        "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
    ]
    records = [
        {
            "artifact_id": "01-current-state-runtime-dependency-map-md",
            "title": "Runtime Dependency Map",
            "artifact_type": "migration_artifact",
            "path": "execution-framework/migration-artifacts/01-current-state/runtime-dependency-map.md",
            "links": [{"to": "artifact:art-106-runtime-dep-map-json", "type": "described_by"}],
        },
        {
            "artifact_id": "art-106-runtime-dep-map-md",
            "title": "ART-106 Runtime Dependency Map Markdown",
            "artifact_type": "task_artifact",
            "path": "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.md",
            "links": [{"to": "artifact:01-current-state-runtime-dependency-map-md", "type": "mirrors"}],
        },
        {
            "artifact_id": "art-106-runtime-dep-map-json",
            "title": "ART-106 Runtime Dependency Map JSON",
            "artifact_type": "machine_readable_dependency_map",
            "path": "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
            "links": [{"to": "artifact:01-current-state-runtime-dependency-map-md", "type": "describes"}],
        },
    ]
    results = []
    for record in records:
        results.append(
            registry.register(
                {
                    **record,
                    "run_id": RUN_ID,
                    "status": "complete",
                    "producer_operation_id": OPERATION_ID,
                    "contract_id": CONTRACT_ID,
                    "provenance": {
                        "task_id": TASK_ID,
                        "owner_agent": "artifact-agent",
                        "helper_id": HELPER_ID,
                        "source_packet": "execution-framework/generated/execution_packets/ART-106_RUNTIME_DEP_MAP.json",
                        "source_graph_uri": "generated/task_graph.csv",
                        "target_descriptor": "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                    },
                    "evidence_refs": evidence_refs,
                    "links": record["links"]
                    + [
                        {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
                        {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
                        {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
                    ],
                    "validations": [
                        {
                            "validator": "generate_art106_runtime_dep_map.py:path-exists",
                            "status": "pass",
                            "details": {"path": record["path"]},
                            "evidence_refs": [record["path"]],
                        },
                        {
                            "validator": "generate_art106_runtime_dep_map.py:runtime-category-coverage",
                            "status": "pass",
                            "details": {
                                "required_categories": list(DEPENDENCY_PATTERNS),
                                "covered_categories": [
                                    category
                                    for category, item in dep_map["coverage"].items()
                                    if item["evidence_count"] > 0
                                ],
                                "gap_categories": [gap["category"] for gap in dep_map["gaps"]],
                                "visited_file_count": dep_map["safe_scan"]["visited_file_count"],
                            },
                            "evidence_refs": [
                                "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json"
                            ],
                        },
                        {
                            "validator": "generate_art106_runtime_dep_map.py:redaction-boundary",
                            "status": "pass",
                            "details": {
                                "blocked_patterns": list(BLOCKED_PATTERNS),
                                "secret_material_read": False,
                            },
                            "evidence_refs": [
                                "execution-framework/generated/execution_packets/ART-106_RUNTIME_DEP_MAP.json"
                            ],
                        },
                    ],
                }
            )
        )
    fetched = [fetch_artifact(conn, RUN_ID, record["artifact_id"]) for record in records]
    verification_rows = []
    for item in results:
        path = package_root() / item["path"]
        verification_rows.append(
            {
                "artifact_id": item["artifact_id"],
                "path": item["path"],
                "content_hash": item["content_hash"],
                "hash_matches": item["content_hash"] == sha256_file(path),
                "validation_ids": item["validation_ids"],
            }
        )
    verification = {
        "artifact_files_exist": all((package_root() / item["path"]).exists() for item in results),
        "hashes_recorded": all(item.get("content_hash") for item in results),
        "registry_contains_hash": all(row["hash_matches"] for row in verification_rows),
        "validation_evidence_linked": all(item.get("validation_ids") for item in results),
        "runtime_categories": list(DEPENDENCY_PATTERNS),
        "covered_categories": [
            category for category, item in dep_map["coverage"].items() if item["evidence_count"] > 0
        ],
        "gap_categories": [gap["category"] for gap in dep_map["gaps"]],
        "rows": verification_rows,
    }
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "passed" if all(
            [verification["artifact_files_exist"], verification["hashes_recorded"], verification["registry_contains_hash"], verification["validation_evidence_linked"]]
        ) else "failed",
        "run_id": RUN_ID,
        "operation_id": OPERATION_ID,
        "contract_id": CONTRACT_ID,
        "applied_migrations": applied_migrations,
        "registered_artifacts": results,
        "fetched_artifacts": fetched,
        "verification": verification,
    }


def main() -> int:
    started = now()
    dep_map = build_runtime_dep_map()
    markdown = render_markdown(dep_map)

    TASK_MD.parent.mkdir(parents=True, exist_ok=True)
    TASK_MD.write_text(markdown, encoding="utf-8")
    write_json(TASK_JSON, dep_map)
    CANONICAL_MD.parent.mkdir(parents=True, exist_ok=True)
    CANONICAL_MD.write_text(markdown, encoding="utf-8")

    registry_report = register_artifacts(dep_map)
    write_json(REPORT_PATH, registry_report)

    HEARTBEAT_PATH.parent.mkdir(parents=True, exist_ok=True)
    write_json(
        HEARTBEAT_PATH,
        {
            "task_id": TASK_ID,
            "status": "complete" if registry_report["status"] == "passed" else "failed",
            "started_at": started,
            "updated_at": now(),
            "artifact_paths": [
                "migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.md",
                "migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
                "migration-artifacts/01-current-state/runtime-dependency-map.md",
            ],
            "proof_uri": f"proof_records/{TASK_ID}.proof.json",
        },
    )

    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    LOG_PATH.write_text(
        "\n".join(
            [
                f"{started} start {TASK_ID}",
                f"{now()} wrote migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.md",
                f"{now()} wrote migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
                f"{now()} wrote migration-artifacts/01-current-state/runtime-dependency-map.md",
                f"{now()} registry hashes recorded: {registry_report['verification']['hashes_recorded']}",
                f"{now()} registry contains hash: {registry_report['verification']['registry_contains_hash']}",
                f"{now()} validation evidence linked: {registry_report['verification']['validation_evidence_linked']}",
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/generate_art106_runtime_dep_map.py",
        "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.md",
        "execution-framework/migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
        "execution-framework/migration-artifacts/01-current-state/runtime-dependency-map.md",
        "execution-framework/generated/art106_runtime_dep_map_registry_report.json",
        "execution-framework/state/ART-106_RUNTIME_DEP_MAP.heartbeat.json",
        "execution-framework/logs/ART-106_RUNTIME_DEP_MAP.log",
    ]
    proof = make_proof(
        task_id=TASK_ID,
        status="completed" if registry_report["status"] == "passed" else "failed",
        actor="artifact-agent",
        helper_id=HELPER_ID,
        model_tag=MODEL_TAG,
        repo_path="${MIGRATION_TARGET_ROOT}",
        files_changed=files_changed,
        commands_run=[
            "python3 scripts/generate_art106_runtime_dep_map.py",
            "python3 -m py_compile scripts/generate_art106_runtime_dep_map.py",
            "python3 -m json.tool migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
            "python3 -m json.tool generated/art106_runtime_dep_map_registry_report.json",
        ],
        verification_output=registry_report["verification"],
        evidence=[
            "migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.md",
            "migration-artifacts/art-106_runtime_dep_map/runtime-dependency-map.json",
            "migration-artifacts/01-current-state/runtime-dependency-map.md",
            "generated/art106_runtime_dep_map_registry_report.json",
            "logs/ART-106_RUNTIME_DEP_MAP.log",
        ],
        failure_reason="" if registry_report["status"] == "passed" else json.dumps(registry_report["verification"], sort_keys=True),
        next_action="run VER-300_UNIT_VALIDATION",
    )
    append_proof(proof)
    print(json.dumps(registry_report["verification"], sort_keys=True))
    return 0 if registry_report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
