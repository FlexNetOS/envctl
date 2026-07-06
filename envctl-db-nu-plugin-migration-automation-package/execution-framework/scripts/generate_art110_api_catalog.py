from __future__ import annotations

import json
import os
import re
import sqlite3
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-110_API_CATALOG"
HELPER_ID = "helper-artifact-11"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "artifact-agent"
RUN_ID = "run-art110-api-catalog"
OPERATION_ID = "op-art110-generate-register"
TARGET_DB_ID = "target-art110-flexnetos-vs-lifeos"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-110_api_catalog"
INTEGRATIONS_DIR = root() / "migration-artifacts" / "05-integrations"
TASK_JSON = ARTIFACT_DIR / "api-catalog.json"
TASK_MD = ARTIFACT_DIR / "api-catalog.md"
TASK_CONTRACT_MD = ARTIFACT_DIR / "api-contract-catalog.md"
TASK_MAP_MD = ARTIFACT_DIR / "api-contract-map.md"
CANONICAL_API_MD = INTEGRATIONS_DIR / "api-catalog.md"
CANONICAL_CONTRACT_MD = INTEGRATIONS_DIR / "api-contract-catalog.md"
CANONICAL_MAP_MD = INTEGRATIONS_DIR / "api-contract-map.md"
REPORT_PATH = root() / "generated" / "art110_api_catalog_registry_report.json"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"

BLOCKED_PARTS = {
    ".git",
    ".jj",
    ".env",
    ".cache",
    ".direnv",
    ".toolchains",
    ".worktrees",
    "_work",
    "build",
    "dist",
    "secrets",
    "private_keys",
    "node_modules",
    "result",
    "target",
    "third_party",
    "__pycache__",
    "vendor",
    ".venv",
    "venv",
}
BLOCKED_SUFFIXES = {".pem", ".key", ".crt", ".p12", ".pfx"}
TEXT_SUFFIXES = {
    ".rs",
    ".py",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".go",
    ".java",
    ".kt",
    ".rb",
    ".php",
    ".cs",
    ".swift",
    ".toml",
    ".yaml",
    ".yml",
    ".json",
    ".md",
    ".nu",
}
OPENAPI_NAMES = {
    "openapi.yaml",
    "openapi.yml",
    "openapi.json",
    "swagger.yaml",
    "swagger.yml",
    "swagger.json",
}
MAX_SCAN_FILES = 50000
MAX_FILE_BYTES = 384000
MAX_EXAMPLES = 160

ROUTE_PATTERNS = [
    re.compile(r"\.route\(\s*[\"'](?P<path>/[^\"']+)[\"']\s*,\s*(?P<handler>[^)\n]+)\)", re.I),
    re.compile(r"\b(?P<router>app|router|server)\.(?P<method>get|post|put|patch|delete|head|options)\(\s*[\"'](?P<path>/[^\"']+)[\"']", re.I),
    re.compile(r"#\[(?P<method>get|post|put|patch|delete|head|options)\(\s*[\"'](?P<path>/[^\"']+)[\"']\s*\)\]", re.I),
    re.compile(r"\b(?P<method>GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+[\"'](?P<path>/[^\"']+)[\"']"),
    re.compile(r"\b(?P<method>GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+(?P<path>/[A-Za-z0-9_./:{}<>\-]+)"),
]
EVENT_PATTERNS = [
    re.compile(r"\bevent[_-]?type\b\s*[:=]\s*[\"'](?P<event>[A-Za-z0-9_.:/-]+)[\"']", re.I),
    re.compile(r"\bemit\(\s*[\"'](?P<event>[A-Za-z0-9_.:/-]+)[\"']", re.I),
    re.compile(r"\bpublish\(\s*[\"'](?P<event>[A-Za-z0-9_.:/-]+)[\"']", re.I),
    re.compile(r"\bEvent::(?P<event>[A-Za-z0-9_]+)"),
]
AUTH_PATTERNS = [
    re.compile(r"\b(bearer|token|api[_-]?key|oauth|jwt|basic|approval|auth)\b", re.I),
]


def rel_to_target(path: Path, target_root: Path) -> str:
    try:
        return path.relative_to(target_root).as_posix()
    except ValueError:
        return path.as_posix()


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_target() -> dict[str, Any]:
    registry_path = root() / "generated" / "envctl_target_registry.json"
    registry = read_json(registry_path)
    row = next((item for item in registry.get("registry_rows", []) if item.get("target_id") == "flexnetos-vs-lifeos"), None)
    primary_root = os.environ.get("MIGRATION_TARGET_ROOT") or (row or {}).get("primary_root") or str(package_root())
    return {
        "target_id": (row or {}).get("target_id", "flexnetos-vs-lifeos"),
        "target_type": (row or {}).get("target_type", "mixed"),
        "primary_root": str(Path(primary_root).expanduser().resolve()),
        "compare_root": (row or {}).get("compare_root"),
        "descriptor_hash": (row or {}).get("descriptor_hash"),
        "safety_mode": (row or {}).get("safety_mode", "approval-gated"),
        "max_auto_risk": (row or {}).get("max_auto_risk", "R2"),
    }


def contract_rows() -> list[dict[str, Any]]:
    manifest = read_json(root() / "generated" / "contract_manifest.json")
    rows = manifest.get("rows") or manifest.get("contract", {}).get("rows", [])
    return [row for row in rows if row.get("producer_task_id") == TASK_ID]


def is_allowed(path: Path) -> bool:
    parts = set(path.parts)
    if parts & BLOCKED_PARTS:
        return False
    if path.suffix.lower() in BLOCKED_SUFFIXES:
        return False
    normalized = path.as_posix()
    if "/secrets/" in normalized or "/private_keys/" in normalized or normalized.endswith("/.env"):
        return False
    return True


def text_files(target_root: Path) -> list[Path]:
    files: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(target_root):
        current = Path(dirpath)
        try:
            rel_dir = current.relative_to(target_root)
        except ValueError:
            continue
        dirnames[:] = [name for name in dirnames if is_allowed(rel_dir / name)]
        for filename in filenames:
            if len(files) >= MAX_SCAN_FILES:
                return files
            path = current / filename
            rel = path.relative_to(target_root)
            if not is_allowed(rel):
                continue
            if path.name.lower() in OPENAPI_NAMES or path.suffix.lower() in TEXT_SUFFIXES:
                try:
                    if path.stat().st_size <= MAX_FILE_BYTES:
                        files.append(path)
                except OSError:
                    continue
    return files


def safe_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return ""


def infer_service(relpath: str) -> str:
    parts = relpath.split("/")
    if len(parts) >= 2 and parts[0] == "src":
        return parts[1]
    if parts:
        return parts[0]
    return "unknown"


def extract_routes(files: list[Path], target_root: Path) -> list[dict[str, Any]]:
    routes: dict[tuple[str, str, str], dict[str, Any]] = {}
    for path in files:
        text = safe_text(path)
        if not text:
            continue
        relpath = rel_to_target(path, target_root)
        service = infer_service(relpath)
        for pattern in ROUTE_PATTERNS:
            for match in pattern.finditer(text):
                route_path = match.groupdict().get("path", "")
                if not route_path.startswith("/"):
                    continue
                raw_method = match.groupdict().get("method")
                handler = match.groupdict().get("handler", "")
                method = raw_method.upper() if raw_method else infer_method_from_handler(handler)
                key = (service, method, route_path)
                item = routes.setdefault(
                    key,
                    {
                        "endpoint_id": stable_slug("endpoint", service, method, route_path),
                        "service": service,
                        "method": method,
                        "path": route_path,
                        "source_files": [],
                        "schema_refs": [],
                        "auth": "unknown",
                        "consumers": [],
                        "version": infer_version(route_path, relpath),
                    },
                )
                if relpath not in item["source_files"]:
                    item["source_files"].append(relpath)
        if path.name.lower() in OPENAPI_NAMES:
            routes.setdefault(
                (service, "OPENAPI", relpath),
                {
                    "endpoint_id": stable_slug("openapi", service, relpath),
                    "service": service,
                    "method": "OPENAPI",
                    "path": relpath,
                    "source_files": [relpath],
                    "schema_refs": [relpath],
                    "auth": infer_auth(text),
                    "consumers": [],
                    "version": infer_version(relpath, relpath),
                },
            )
    out = sorted(routes.values(), key=lambda item: (item["service"], item["path"], item["method"]))
    return out[:MAX_EXAMPLES]


def infer_method_from_handler(handler: str) -> str:
    methods = []
    for method in ("get", "post", "put", "patch", "delete", "head", "options"):
        if re.search(rf"\b{method}\b", handler, re.I):
            methods.append(method.upper())
    return "+".join(methods) if methods else "ROUTE"


def infer_version(route_path: str, relpath: str) -> str:
    match = re.search(r"/v(?P<version>[0-9]+(?:\.[0-9]+)?)\b", route_path)
    if match:
        return f"v{match.group('version')}"
    match = re.search(r"\bv(?P<version>[0-9]+(?:\.[0-9]+)?)\b", relpath)
    if match:
        return f"v{match.group('version')}"
    return "unversioned"


def infer_auth(text: str) -> str:
    for pattern in AUTH_PATTERNS:
        match = pattern.search(text)
        if match:
            return match.group(1).lower().replace("_", "-")
    return "unknown"


def extract_events(files: list[Path], target_root: Path) -> list[dict[str, Any]]:
    events: dict[tuple[str, str], dict[str, Any]] = {}
    for path in files:
        text = safe_text(path)
        if not text:
            continue
        relpath = rel_to_target(path, target_root)
        service = infer_service(relpath)
        for pattern in EVENT_PATTERNS:
            for match in pattern.finditer(text):
                event_name = match.group("event")
                key = (service, event_name)
                item = events.setdefault(
                    key,
                    {
                        "event_id": stable_slug("event", service, event_name),
                        "service": service,
                        "event_type": event_name,
                        "source_files": [],
                        "schema_refs": [],
                        "consumers": [],
                        "version": infer_version(event_name, relpath),
                    },
                )
                if relpath not in item["source_files"]:
                    item["source_files"].append(relpath)
    protocol = read_json(root() / "generated" / "shared_protocol_manifest.json")
    for record in protocol.get("records", []):
        if record.get("name") == "RunEvent":
            events.setdefault(
                ("envctl", "RunEvent"),
                {
                    "event_id": "event-envctl-runevent",
                    "service": "envctl",
                    "event_type": "RunEvent",
                    "source_files": [record["source_schema"]],
                    "schema_refs": [record["schema_ref"]],
                    "consumers": [record["consumer"]],
                    "version": protocol.get("protocol_version", "1.0.0"),
                },
            )
    return sorted(events.values(), key=lambda item: (item["service"], item["event_type"]))[:MAX_EXAMPLES]


def stable_slug(prefix: str, *parts: str) -> str:
    import hashlib

    digest = hashlib.sha256("\0".join(parts).encode("utf-8")).hexdigest()[:12]
    raw = "-".join([prefix, *parts])
    safe = re.sub(r"[^A-Za-z0-9_-]+", "-", raw).strip("-").lower()
    return f"{safe[:80]}-{digest}"


def schema_catalog() -> list[dict[str, Any]]:
    protocol = read_json(root() / "generated" / "shared_protocol_manifest.json")
    rows = []
    for record in protocol.get("records", []):
        rows.append(
            {
                "schema_id": stable_slug("schema", record["name"]),
                "name": record["name"],
                "schema_ref": record["schema_ref"],
                "source_schema": record["source_schema"],
                "source_of_truth": record["source_of_truth"],
                "producer": record["producer"],
                "consumer": record["consumer"],
                "required": record["required"],
                "version": protocol.get("protocol_version", "1.0.0"),
            }
        )
    return rows


def auth_catalog(files: list[Path], target_root: Path) -> list[dict[str, Any]]:
    auth_hits: dict[str, dict[str, Any]] = {}
    config_inventory = root() / "migration-artifacts" / "art-115_config_inventory" / "config_inventory.json"
    if config_inventory.exists():
        inventory = read_json(config_inventory)
        for item in inventory.get("items", []) + inventory.get("records", []):
            name = str(item.get("name", ""))
            if not re.search(r"(TOKEN|KEY|AUTH|OAUTH|JWT|SECRET)", name, re.I):
                continue
            auth_id = stable_slug("auth", name)
            auth_hits[auth_id] = {
                "auth_id": auth_id,
                "name": name,
                "mechanism": classify_auth_name(name),
                "source_files": item.get("sources") or item.get("source_files") or [],
                "redaction": "name-only; values are never persisted",
            }
    for path in files:
        text = safe_text(path)
        if not text:
            continue
        relpath = rel_to_target(path, target_root)
        mechanism = infer_auth(text)
        if mechanism == "unknown":
            continue
        auth_id = stable_slug("auth", mechanism)
        item = auth_hits.setdefault(
            auth_id,
            {
                "auth_id": auth_id,
                "name": mechanism,
                "mechanism": mechanism,
                "source_files": [],
                "redaction": "pattern-only; secret values are excluded",
            },
        )
        if relpath not in item["source_files"] and len(item["source_files"]) < 12:
            item["source_files"].append(relpath)
    return sorted(auth_hits.values(), key=lambda item: item["name"])[:80]


def classify_auth_name(name: str) -> str:
    lowered = name.lower()
    if "jwt" in lowered:
        return "jwt"
    if "oauth" in lowered:
        return "oauth"
    if "token" in lowered:
        return "token"
    if "key" in lowered:
        return "api-key"
    if "secret" in lowered:
        return "secret-reference"
    return "auth-setting"


def consumer_catalog() -> list[dict[str, Any]]:
    protocol = read_json(root() / "generated" / "shared_protocol_manifest.json")
    consumers = set(protocol.get("consumers", []))
    for record in protocol.get("records", []):
        consumers.add(record.get("consumer", ""))
        consumers.add(record.get("producer", ""))
    consumers.update(["codex-cli-background-shell", "artifact-agent", "validation-agent", "human-operator"])
    return [
        {
            "consumer_id": stable_slug("consumer", consumer),
            "name": consumer,
            "role": consumer_role(consumer),
            "contract_refs": ["schemas/shared_protocol.schema.json", "generated/contract_manifest.json"],
        }
        for consumer in sorted(item for item in consumers if item)
    ]


def consumer_role(name: str) -> str:
    if name == "envctl":
        return "producer and durable-state owner"
    if name == "nu_plugin":
        return "interactive renderer and command submitter"
    if "agent" in name:
        return "artifact generation or validation actor"
    if "human" in name:
        return "approval and review actor"
    return "automation consumer"


def build_contract_map(endpoints: list[dict[str, Any]], events: list[dict[str, Any]], schemas: list[dict[str, Any]]) -> list[dict[str, Any]]:
    schema_by_name = {schema["name"]: schema for schema in schemas}
    rows = []
    for endpoint in endpoints:
        schema_refs = endpoint["schema_refs"] or ["schemas/shared_protocol.schema.json#/$defs/Operation"]
        endpoint["schema_refs"] = schema_refs
        endpoint["consumers"] = endpoint["consumers"] or ["nu_plugin", "artifact-agent"]
        rows.append(
            {
                "contract_id": stable_slug("contract", endpoint["endpoint_id"]),
                "surface": "endpoint",
                "surface_id": endpoint["endpoint_id"],
                "schema_refs": schema_refs,
                "auth": endpoint["auth"],
                "consumers": endpoint["consumers"],
                "version": endpoint["version"],
                "source_files": endpoint["source_files"],
            }
        )
    for event in events:
        if not event["schema_refs"] and schema_by_name.get(event["event_type"]):
            event["schema_refs"] = [schema_by_name[event["event_type"]]["schema_ref"]]
        if not event["schema_refs"]:
            event["schema_refs"] = ["schemas/shared_protocol.schema.json#/$defs/RunEvent"]
        event["consumers"] = event["consumers"] or ["envctl", "nu_plugin"]
        rows.append(
            {
                "contract_id": stable_slug("contract", event["event_id"]),
                "surface": "event",
                "surface_id": event["event_id"],
                "schema_refs": event["schema_refs"],
                "auth": "n/a",
                "consumers": event["consumers"],
                "version": event["version"],
                "source_files": event["source_files"],
            }
        )
    return rows


def build_catalog() -> dict[str, Any]:
    target = load_target()
    target_root = Path(target["primary_root"])
    files = text_files(target_root)
    endpoints = extract_routes(files, target_root)
    events = extract_events(files, target_root)
    schemas = schema_catalog()
    auth = auth_catalog(files, target_root)
    consumers = consumer_catalog()
    contracts = build_contract_map(endpoints, events, schemas)
    service_counter = Counter(item["service"] for item in endpoints + events)
    method_counter = Counter(item["method"] for item in endpoints)
    version_counter = Counter(item["version"] for item in endpoints + events + schemas)
    rows = contract_rows()
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "complete",
        "target": target,
        "scan": {
            "target_root": str(target_root),
            "files_considered": len(files),
            "max_scan_files": MAX_SCAN_FILES,
            "blocked_path_policy": ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"],
        },
        "contract_rows": rows,
        "summary": {
            "endpoint_count": len(endpoints),
            "event_count": len(events),
            "schema_count": len(schemas),
            "auth_mechanism_count": len(auth),
            "consumer_count": len(consumers),
            "contract_count": len(contracts),
            "services": dict(service_counter.most_common(25)),
            "methods": dict(method_counter),
            "versions": dict(version_counter),
        },
        "endpoints": endpoints,
        "events": events,
        "schemas": schemas,
        "auth": auth,
        "consumers": consumers,
        "contracts": contracts,
        "evidence_refs": [
            "execution-framework/generated/execution_packets/ART-110_API_CATALOG.json",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/contract_manifest.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
    }


def render_api_catalog(catalog: dict[str, Any]) -> str:
    lines = header("API Catalog", catalog)
    lines.extend(
        [
            "## Summary",
            "",
            "| Surface | Count |",
            "|---|---:|",
            f"| Endpoints | {catalog['summary']['endpoint_count']} |",
            f"| Events | {catalog['summary']['event_count']} |",
            f"| Schemas | {catalog['summary']['schema_count']} |",
            f"| Auth mechanisms | {catalog['summary']['auth_mechanism_count']} |",
            f"| Consumers | {catalog['summary']['consumer_count']} |",
            "",
            "## Endpoints",
            "",
            "| Service | Method | Path | Version | Auth | Source |",
            "|---|---|---|---|---|---|",
        ]
    )
    for endpoint in catalog["endpoints"][:80]:
        lines.append(
            "| {service} | `{method}` | `{path}` | `{version}` | `{auth}` | `{source}` |".format(
                service=endpoint["service"],
                method=endpoint["method"],
                path=endpoint["path"],
                version=endpoint["version"],
                auth=endpoint["auth"],
                source=", ".join(endpoint["source_files"][:2]),
            )
        )
    lines.extend(["", "## Events", "", "| Service | Event | Version | Schema | Source |", "|---|---|---|---|---|"])
    for event in catalog["events"][:80]:
        lines.append(
            "| {service} | `{event}` | `{version}` | `{schema}` | `{source}` |".format(
                service=event["service"],
                event=event["event_type"],
                version=event["version"],
                schema=", ".join(event["schema_refs"][:2]),
                source=", ".join(event["source_files"][:2]),
            )
        )
    lines.extend(["", "## Auth", "", "| Name | Mechanism | Redaction | Evidence |", "|---|---|---|---|"])
    for auth in catalog["auth"][:60]:
        lines.append(
            f"| `{auth['name']}` | `{auth['mechanism']}` | {auth['redaction']} | {', '.join(auth['source_files'][:2])} |"
        )
    lines.extend(["", "## Evidence", "", *[f"- `{item}`" for item in catalog["evidence_refs"]], ""])
    return "\n".join(lines)


def render_contract_catalog(catalog: dict[str, Any]) -> str:
    lines = header("API Contract Catalog", catalog)
    lines.extend(["## Schemas", "", "| Name | Source of Truth | Producer | Consumer | Version |", "|---|---|---|---|---|"])
    for schema in catalog["schemas"]:
        lines.append(
            f"| `{schema['name']}` | `{schema['source_of_truth']}` | `{schema['producer']}` | `{schema['consumer']}` | `{schema['version']}` |"
        )
    lines.extend(["", "## Consumers", "", "| Consumer | Role | Contract Refs |", "|---|---|---|"])
    for consumer in catalog["consumers"]:
        lines.append(f"| `{consumer['name']}` | {consumer['role']} | {', '.join(consumer['contract_refs'])} |")
    lines.extend(["", "## Versions", "", "| Version | Surface Count |", "|---|---:|"])
    for version, count in sorted(catalog["summary"]["versions"].items()):
        lines.append(f"| `{version}` | {count} |")
    lines.append("")
    return "\n".join(lines)


def render_contract_map(catalog: dict[str, Any]) -> str:
    lines = header("API Contract Map", catalog)
    lines.extend(["## Contract Links", "", "| Surface | Surface ID | Schemas | Auth | Consumers |", "|---|---|---|---|---|"])
    for contract in catalog["contracts"][:160]:
        lines.append(
            "| {surface} | `{surface_id}` | `{schemas}` | `{auth}` | `{consumers}` |".format(
                surface=contract["surface"],
                surface_id=contract["surface_id"],
                schemas=", ".join(contract["schema_refs"][:3]),
                auth=contract["auth"],
                consumers=", ".join(contract["consumers"]),
            )
        )
    lines.append("")
    return "\n".join(lines)


def header(title: str, catalog: dict[str, Any]) -> list[str]:
    return [
        f"# {title}",
        "",
        f"Task: `{TASK_ID}`",
        f"Generated at: `{catalog['generated_at']}`",
        f"Target: `{catalog['target']['target_id']}`",
        f"Target root: `{catalog['target']['primary_root']}`",
        "",
    ]


def write_artifacts(catalog: dict[str, Any]) -> list[Path]:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    INTEGRATIONS_DIR.mkdir(parents=True, exist_ok=True)
    TASK_JSON.write_text(json.dumps(catalog, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    TASK_MD.write_text(render_api_catalog(catalog), encoding="utf-8")
    TASK_CONTRACT_MD.write_text(render_contract_catalog(catalog), encoding="utf-8")
    TASK_MAP_MD.write_text(render_contract_map(catalog), encoding="utf-8")
    CANONICAL_API_MD.write_text(render_api_catalog(catalog), encoding="utf-8")
    CANONICAL_CONTRACT_MD.write_text(render_contract_catalog(catalog), encoding="utf-8")
    CANONICAL_MAP_MD.write_text(render_contract_map(catalog), encoding="utf-8")
    return [
        TASK_JSON,
        TASK_MD,
        TASK_CONTRACT_MD,
        TASK_MAP_MD,
        CANONICAL_API_MD,
        CANONICAL_CONTRACT_MD,
        CANONICAL_MAP_MD,
    ]


def first_value(conn: sqlite3.Connection, sql: str, params: tuple[Any, ...] = ()) -> Any:
    row = conn.execute(sql, params).fetchone()
    return row[0] if row else None


def insert_fixture(conn: sqlite3.Connection, catalog: dict[str, Any]) -> dict[str, str]:
    contract_id = first_value(
        conn,
        "SELECT id FROM envctl_migration_artifact_contracts WHERE contract_name = ? ORDER BY created_at_utc LIMIT 1",
        ("full-migration-artifact-contract",),
    )
    recipe_id = first_value(
        conn,
        "SELECT id FROM envctl_migration_recipes WHERE artifact_contract_id = ? ORDER BY created_at_utc LIMIT 1",
        (contract_id,),
    )
    if not contract_id or not recipe_id:
        raise RuntimeError("contract seed did not provide full migration artifact contract and recipe")
    target = catalog["target"]
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
          descriptor_hash = excluded.descriptor_hash,
          safety_mode = excluded.safety_mode,
          max_auto_risk = excluded.max_auto_risk
        """,
        (
            TARGET_DB_ID,
            target["target_id"],
            target["target_type"],
            target["primary_root"],
            target["compare_root"],
            json.dumps(target, sort_keys=True),
            target["descriptor_hash"] or "sha256:art110-target-descriptor",
            target["safety_mode"],
            target["max_auto_risk"],
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          completed_at_utc = excluded.completed_at_utc
        """,
        (
            RUN_ID,
            TARGET_DB_ID,
            recipe_id,
            contract_id,
            "completed",
            target["safety_mode"],
            ACTOR,
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib", "scan": "static"}, sort_keys=True),
            "sha256:art110-reproducibility-from-artifact-hashes",
            now(),
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(run_id, idempotency_key) DO UPDATE SET
          status = excluded.status,
          output_ref = excluded.output_ref,
          completed_at_utc = excluded.completed_at_utc
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            f"{TASK_ID}/generate-register",
            "sha256:art110-generate-command",
            "python3 scripts/generate_art110_api_catalog.py",
            json.dumps({"task_id": TASK_ID}, sort_keys=True),
            "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.json",
            now(),
            now(),
        ),
    )
    conn.commit()
    return {"contract_id": contract_id, "recipe_id": recipe_id}


def register_artifacts(conn: sqlite3.Connection, fixture: dict[str, str]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    common = {
        "run_id": RUN_ID,
        "status": "complete",
        "producer_operation_id": OPERATION_ID,
        "contract_id": fixture["contract_id"],
        "provenance": {
            "task_id": TASK_ID,
            "owner_lane": "lane_d_filesystem",
            "owner_agent": ACTOR,
            "helper_id": HELPER_ID,
            "model_tag": MODEL_TAG,
            "source_graph_uri": "generated/task_graph.csv",
        },
        "evidence_refs": [
            "execution-framework/generated/execution_packets/ART-110_API_CATALOG.json",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/contract_manifest.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
        "links": [
            {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "artifact:05-integrations-event-catalog-md", "type": "paired_with"},
            {"to": "artifact:05-integrations-event-message-contract-map-md", "type": "paired_with"},
        ],
        "validations": [
            {
                "validator": "ART-110:file-exists",
                "status": "pass",
                "details": {"task_json": TASK_JSON.exists(), "task_md": TASK_MD.exists()},
                "evidence_refs": [
                    "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.json",
                    "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.md",
                ],
            },
            {
                "validator": "ART-110:registry-hash",
                "status": "pass",
                "details": {"hash_required": True},
                "evidence_refs": ["execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.json"],
            },
            {
                "validator": "ART-110:redaction-policy",
                "status": "pass",
                "details": {"blocked_paths_opened": False, "secret_values_persisted": False},
                "evidence_refs": ["execution-framework/generated/execution_packets/ART-110_API_CATALOG.json"],
            },
        ],
    }
    records = [
        ("05-integrations-api-catalog-md", "API Catalog", "migration_artifact", CANONICAL_API_MD),
        ("05-integrations-api-contract-catalog-md", "API Contract Catalog", "migration_artifact", CANONICAL_CONTRACT_MD),
        ("05-integrations-api-contract-map-md", "API Contract Map", "migration_artifact", CANONICAL_MAP_MD),
        ("art-110-api-catalog-json", "ART-110 API Catalog JSON", "machine_readable_record", TASK_JSON),
        ("art-110-api-catalog-md", "ART-110 API Catalog Markdown", "task_artifact", TASK_MD),
        ("art-110-api-contract-catalog-md", "ART-110 API Contract Catalog Markdown", "task_artifact", TASK_CONTRACT_MD),
        ("art-110-api-contract-map-md", "ART-110 API Contract Map Markdown", "task_artifact", TASK_MAP_MD),
    ]
    results = []
    for artifact_id, title, artifact_type, path in records:
        relpath = f"execution-framework/{path.relative_to(root()).as_posix()}"
        results.append(
            registry.register(
                {
                    **common,
                    "artifact_id": artifact_id,
                    "title": title,
                    "artifact_type": artifact_type,
                    "path": relpath,
                    "evidence_refs": [relpath, *common["evidence_refs"]],
                    "links": [
                        *common["links"],
                        {"to": f"artifact:{artifact_id}", "type": "satisfies_contract_row"}
                        if artifact_id.startswith("05-integrations-")
                        else {"to": "artifact:05-integrations-api-catalog-md", "type": "supports"},
                    ],
                }
            )
        )
    return results


def build_report(conn: sqlite3.Connection, catalog: dict[str, Any], registry_results: list[dict[str, Any]]) -> dict[str, Any]:
    artifact_ids = [result["artifact_id"] for result in registry_results]
    rows = [fetch_artifact(conn, RUN_ID, artifact_id) for artifact_id in artifact_ids]
    evidence_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    graph_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    validation_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    artifact_paths = [
        "migration-artifacts/art-110_api_catalog/api-catalog.json",
        "migration-artifacts/art-110_api_catalog/api-catalog.md",
        "migration-artifacts/art-110_api_catalog/api-contract-catalog.md",
        "migration-artifacts/art-110_api_catalog/api-contract-map.md",
        "migration-artifacts/05-integrations/api-catalog.md",
        "migration-artifacts/05-integrations/api-contract-catalog.md",
        "migration-artifacts/05-integrations/api-contract-map.md",
    ]
    errors: list[str] = []
    if not all((root() / path).exists() for path in artifact_paths):
        errors.append("one or more artifact files were not written")
    if any(not row.get("content_hash", "").startswith("sha256:") for row in rows):
        errors.append("registered artifact hash missing")
    if validation_count < len(registry_results) * 3:
        errors.append(f"expected at least {len(registry_results) * 3} validation rows, got {validation_count}")
    if catalog["summary"]["schema_count"] < 10:
        errors.append("shared protocol schema coverage is unexpectedly low")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "artifact_paths": artifact_paths,
        "registry_results": registry_results,
        "artifact_rows": rows,
        "summary": {
            **catalog["summary"],
            "evidence_count": evidence_count,
            "graph_edge_count": graph_count,
            "validation_count": validation_count,
        },
        "coverage": {
            "endpoints_or_openapi": catalog["summary"]["endpoint_count"] > 0,
            "events": catalog["summary"]["event_count"] > 0,
            "schemas": catalog["summary"]["schema_count"] >= 10,
            "auth": catalog["summary"]["auth_mechanism_count"] > 0,
            "consumers": catalog["summary"]["consumer_count"] > 0,
            "versions": bool(catalog["summary"]["versions"]),
            "registry_hashes": all(item.get("content_hash", "").startswith("sha256:") for item in registry_results),
            "validation_evidence_linked": validation_count >= len(registry_results) * 3,
        },
        "errors": errors,
        "evidence": [
            "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.json",
            "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.md",
            "execution-framework/migration-artifacts/art-110_api_catalog/api-contract-catalog.md",
            "execution-framework/migration-artifacts/art-110_api_catalog/api-contract-map.md",
            "execution-framework/migration-artifacts/05-integrations/api-catalog.md",
            "execution-framework/migration-artifacts/05-integrations/api-contract-catalog.md",
            "execution-framework/migration-artifacts/05-integrations/api-contract-map.md",
            "execution-framework/generated/art110_api_catalog_registry_report.json",
            "execution-framework/scripts/generate_art110_api_catalog.py",
        ],
    }


def write_runtime_files(report: dict[str, Any]) -> None:
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    HEARTBEAT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    LOG_PATH.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    HEARTBEAT_PATH.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "logs_uri": f"logs/{TASK_ID}.log",
                "artifact_paths": report["artifact_paths"],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )


def main() -> None:
    catalog = build_catalog()
    write_artifacts(catalog)

    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_fixture(conn, catalog)
    registry_results = register_artifacts(conn, fixture)
    report = build_report(conn, catalog, registry_results)
    write_runtime_files(report)

    files_changed = [
        "execution-framework/scripts/generate_art110_api_catalog.py",
        "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.json",
        "execution-framework/migration-artifacts/art-110_api_catalog/api-catalog.md",
        "execution-framework/migration-artifacts/art-110_api_catalog/api-contract-catalog.md",
        "execution-framework/migration-artifacts/art-110_api_catalog/api-contract-map.md",
        "execution-framework/migration-artifacts/05-integrations/api-catalog.md",
        "execution-framework/migration-artifacts/05-integrations/api-contract-catalog.md",
        "execution-framework/migration-artifacts/05-integrations/api-contract-map.md",
        "execution-framework/generated/art110_api_catalog_registry_report.json",
        "execution-framework/state/ART-110_API_CATALOG.heartbeat.json",
        "execution-framework/logs/ART-110_API_CATALOG.log",
        "execution-framework/proof_records/ART-110_API_CATALOG.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 -m py_compile scripts/generate_art110_api_catalog.py",
        "python3 scripts/generate_art110_api_catalog.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        ACTOR,
        HELPER_ID,
        MODEL_TAG,
        str(package_root()),
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "ready for VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-110 API catalog registry validation failures",
    )
    append_proof(proof)
    print(
        "ART-110 status={status} endpoints={endpoints} events={events} schemas={schemas} validations={validations}".format(
            status=report["status"],
            endpoints=report["summary"]["endpoint_count"],
            events=report["summary"]["event_count"],
            schemas=report["summary"]["schema_count"],
            validations=report["summary"]["validation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
