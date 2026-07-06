from __future__ import annotations

import fnmatch
import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

from _common import package_root, root, sha256_file


REDACTION_TOKEN = "<REDACTED:SECRET>"

DEFAULT_BLOCKED_PATHS = [
    "**/.env",
    "**/secrets/**",
    "**/private_keys/**",
    "**/*.pem",
    "**/*.key",
]

SCAN_SURFACES = [
    "execution-framework/logs/**/*.log",
    "execution-framework/proof_records/**/*.json",
    "execution-framework/generated/execution_packets/**/*.json",
    "execution-framework/generated/**/*scan*.json",
    "execution-framework/generated/**/*manifest*.json",
    "execution-framework/generated/**/*validation*.json",
    "execution-framework/state/**/*.json",
]

APPROVED_EVIDENCE_LOCATIONS = [
    "execution-framework/logs/",
    "execution-framework/proof_records/",
    "execution-framework/generated/",
]

REPRODUCIBILITY_IDENTITY_FIELDS = [
    "target_descriptor_hash",
    "artifact_contract_hash",
    "recipe_hash",
    "package_hashes",
    "collector_versions",
    "tool_versions",
    "operation_input_hashes",
    "evidence_hashes",
    "artifact_hashes",
    "approval_decision_hashes",
]

REPLAY_MODES = {
    "verify-only": "Recompute hashes and confirm artifacts/evidence still match.",
    "dry-run-plan": "Reconstruct operation plan without executing target-affecting commands.",
    "execute-safe": "Re-run R0-R2 operations.",
    "execute-full": "Requires explicit approval for R3+.",
}

SUPPLEMENTAL_PATTERNS = [
    {
        "id": "authorization-bearer-header",
        "regex": r"(?i)\bauthorization\s*[:=]\s*bearer\s+[A-Za-z0-9._~+/=-]+",
    },
    {
        "id": "generic-secret-assignment",
        "regex": (
            r"(?i)\b(api[_-]?key|secret|token|password|passwd|private[_-]?key|"
            r"authorization|bearer)\b\s*[:=]\s*(\"[^\"]+\"|'[^']+'|[^\s,}]+)"
        ),
    },
    {
        "id": "aws-access-key-id-assignment",
        "regex": r"(?i)\baws_access_key_id\s*=\s*[A-Z0-9]{16,}",
    },
    {
        "id": "aws-secret-access-key-assignment",
        "regex": r"(?i)\baws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{20,}",
    },
    {
        "id": "private-key-pem-block",
        "regex": (
            r"(?is)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?"
            r"-----END [A-Z ]*PRIVATE KEY-----"
        ),
    },
    {
        "id": "openai-style-api-key",
        "regex": r"\bsk-[A-Za-z0-9_-]{20,}\b",
    },
    {
        "id": "github-token",
        "regex": r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b",
    },
]


@dataclass(frozen=True)
class RedactionPattern:
    pattern_id: str
    regex: str
    source: str
    compiled: re.Pattern[str]


def pattern_source_path() -> Path:
    return package_root() / "helpers" / "redaction_patterns.txt"


def load_patterns() -> list[RedactionPattern]:
    patterns: list[RedactionPattern] = []
    for item in SUPPLEMENTAL_PATTERNS:
        patterns.append(
            RedactionPattern(
                pattern_id=item["id"],
                regex=item["regex"],
                source="execution-framework/scripts/redaction_controls.py",
                compiled=re.compile(item["regex"]),
            )
        )
    source = pattern_source_path()
    if source.exists():
        for idx, line in enumerate(source.read_text(encoding="utf-8").splitlines(), start=1):
            regex = line.strip()
            if not regex or regex.startswith("#"):
                continue
            patterns.append(
                RedactionPattern(
                    pattern_id=f"source-{idx}",
                    regex=regex,
                    source=str(source.relative_to(package_root())),
                    compiled=re.compile(regex),
                )
            )
    return patterns


def redact_text(value: str, patterns: Iterable[RedactionPattern] | None = None) -> str:
    redacted = value
    for pattern in patterns or load_patterns():
        redacted = pattern.compiled.sub(REDACTION_TOKEN, redacted)
    return redacted


def redact_value(value: Any, patterns: Iterable[RedactionPattern] | None = None) -> Any:
    active_patterns = list(patterns or load_patterns())
    if isinstance(value, str):
        return redact_text(value, active_patterns)
    if isinstance(value, list):
        return [redact_value(item, active_patterns) for item in value]
    if isinstance(value, dict):
        return {key: redact_value(item, active_patterns) for key, item in value.items()}
    return value


def path_for_match(path: Path) -> str:
    try:
        return str(path.relative_to(package_root()))
    except ValueError:
        return str(path)


def is_blocked_path(path: Path, blocked_globs: Iterable[str] = DEFAULT_BLOCKED_PATHS) -> bool:
    rel = path_for_match(path)
    parts = set(Path(rel).parts)
    for pattern in blocked_globs:
        tail_pattern = pattern[3:] if pattern.startswith("**/") else pattern
        names = {rel, path.name, tail_pattern}
        if any(fnmatch.fnmatch(name, pattern) for name in [rel, path.name]):
            return True
        if pattern.startswith("**/") and any(fnmatch.fnmatch(name, tail_pattern) for name in [rel, path.name]):
            return True
        if tail_pattern in {".env", "*.pem", "*.key"} and fnmatch.fnmatch(path.name, tail_pattern):
            return True
        if tail_pattern in {"secrets/**", "private_keys/**"}:
            directory = tail_pattern.split("/", 1)[0]
            if directory in parts:
                return True
    return False


def iter_scan_files() -> list[Path]:
    base = package_root()
    files: set[Path] = set()
    for surface in SCAN_SURFACES:
        files.update(base.glob(surface))
    return sorted(path for path in files if path.is_file() and not is_blocked_path(path))


def find_secret_markers(path: Path, patterns: Iterable[RedactionPattern] | None = None) -> list[dict[str, Any]]:
    active_patterns = list(patterns or load_patterns())
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return [
            {
                "file": path_for_match(path),
                "pattern_id": "__binary_or_non_utf8__",
                "line": None,
            }
        ]
    findings = []
    for pattern in active_patterns:
        for match in pattern.compiled.finditer(content):
            line = content.count("\n", 0, match.start()) + 1
            findings.append(
                {
                    "file": path_for_match(path),
                    "pattern_id": pattern.pattern_id,
                    "line": line,
                }
            )
    return findings


def sanitize_file(path: Path, patterns: Iterable[RedactionPattern] | None = None) -> bool:
    active_patterns = list(patterns or load_patterns())
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return False
    redacted = redact_text(content, active_patterns)
    if redacted == content:
        return False
    path.write_text(redacted, encoding="utf-8")
    return True


def sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def chained_event_hash(previous_event_hash: str, event: dict[str, Any]) -> str:
    canonical = json.dumps(event, sort_keys=True, separators=(",", ":"))
    return sha256_text(previous_event_hash + canonical)


def build_controls_manifest(generated_files: Iterable[str] = ()) -> dict[str, Any]:
    patterns = load_patterns()
    source_path = pattern_source_path()
    source_hash = sha256_file(source_path) if source_path.exists() else None
    return {
        "schema_version": "1.0",
        "control_name": "envctl_nu_plugin_security_redaction",
        "redaction_token": REDACTION_TOKEN,
        "pattern_source": path_for_match(source_path),
        "pattern_source_sha256": source_hash,
        "pattern_count": len(patterns),
        "patterns": [
            {
                "id": pattern.pattern_id,
                "source": pattern.source,
                "regex_sha256": sha256_text(pattern.regex),
            }
            for pattern in patterns
        ],
        "blocked_paths": DEFAULT_BLOCKED_PATHS,
        "scan_surfaces": SCAN_SURFACES,
        "approved_evidence_locations": APPROVED_EVIDENCE_LOCATIONS,
        "capture_rules": [
            "Redact command strings, log lines, proof fields, scan payloads, and packet payloads before persistence.",
            "Persist raw evidence only under approved evidence locations and only when the evidence record marks redacted=false.",
            "Mark every evidence record with a redacted boolean and sha256 hash.",
            "Hash generated artifacts and evidence files used by proof records.",
            "Require approval for execute-full replay or any R3+ operation.",
            "Do not scan or persist blocked secret-bearing paths.",
        ],
        "evidence_record_required_fields": [
            "evidence_id",
            "run_id",
            "uri",
            "evidence_kind",
            "sha256",
            "redacted",
        ],
        "reproducibility_identity_fields": REPRODUCIBILITY_IDENTITY_FIELDS,
        "replay_modes": REPLAY_MODES,
        "chain_integrity": "event_hash = sha256(previous_event_hash + canonical_event_json)",
        "generated_files": sorted(generated_files),
    }


def write_markdown_doc(path: Path, manifest: dict[str, Any], report: dict[str, Any]) -> None:
    lines = [
        "# Security Redaction And Reproducibility Controls",
        "",
        f"Control: `{manifest['control_name']}`",
        f"Schema version: `{manifest['schema_version']}`",
        f"Validation status: `{report['status']}`",
        "",
        "## Capture Rules",
        "",
    ]
    lines.extend(f"- {rule}" for rule in manifest["capture_rules"])
    lines.extend(
        [
            "",
            "## Scan Surfaces",
            "",
        ]
    )
    lines.extend(f"- `{surface}`" for surface in manifest["scan_surfaces"])
    lines.extend(
        [
            "",
            "## Blocked Paths",
            "",
        ]
    )
    lines.extend(f"- `{pattern}`" for pattern in manifest["blocked_paths"])
    lines.extend(
        [
            "",
            "## Reproducibility Identity",
            "",
        ]
    )
    lines.extend(f"- `{field}`" for field in manifest["reproducibility_identity_fields"])
    lines.extend(
        [
            "",
            "## Replay Modes",
            "",
            "| mode | behavior |",
            "|---|---|",
        ]
    )
    lines.extend(f"| `{mode}` | {behavior} |" for mode, behavior in manifest["replay_modes"].items())
    lines.extend(
        [
            "",
            "## Verification",
            "",
            f"- Files scanned: `{report['files_scanned']}`",
            f"- Pre-redaction findings: `{report['pre_redaction_finding_count']}`",
            f"- Sanitized files: `{report['sanitized_file_count']}`",
            f"- Redaction fixtures passed: `{report['redaction_fixture_count']}`",
            f"- Secret findings: `{report['secret_finding_count']}`",
            f"- Blocked path probes passed: `{report['blocked_path_probe_count']}`",
            f"- Chain hash sample: `{report['chain_hash_sample']}`",
        ]
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
