from __future__ import annotations

import fnmatch
import hashlib
import json
import re
import sqlite3
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


VALIDATION_STATUSES = {"pass", "fail", "warn", "blocked", "unknown"}
EVIDENCE_KINDS = {"reconciliation", "parity", "test_result", "proof_record"}
BLOCKED_PATH_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")


def _stable_id(prefix: str, *parts: str) -> str:
    digest = hashlib.sha256("\0".join(parts).encode("utf-8")).hexdigest()[:24]
    safe_prefix = re.sub(r"[^a-zA-Z0-9_-]+", "-", prefix).strip("-")
    return f"{safe_prefix}-{digest}"


def _json_dumps(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return f"sha256:{h.hexdigest()}"


def _resolve_existing_path(base: Path, relpath: str | None) -> Path | None:
    if not relpath:
        return None
    raw = Path(relpath)
    candidates = [raw] if raw.is_absolute() else [base / raw, base / "execution-framework" / raw]
    for candidate in candidates:
        if candidate.exists() and candidate.is_file():
            return candidate
    return None


@dataclass(frozen=True)
class EvidenceRef:
    uri: str
    evidence_kind: str
    sha256: str | None = None
    redacted: bool = False
    metadata: dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_mapping(cls, raw: dict[str, Any]) -> "EvidenceRef":
        uri = str(raw.get("uri", "")).strip()
        if not uri:
            raise ValueError("evidence ref requires uri")
        evidence_kind = str(raw.get("evidence_kind", "")).strip()
        if evidence_kind not in EVIDENCE_KINDS:
            raise ValueError(f"invalid evidence kind: {evidence_kind}")
        return cls(
            uri=uri,
            evidence_kind=evidence_kind,
            sha256=raw.get("sha256"),
            redacted=bool(raw.get("redacted", False)),
            metadata=dict(raw.get("metadata") or {}),
        )


@dataclass(frozen=True)
class ValidationEvidenceRecord:
    validation_id: str
    run_id: str
    validator: str
    status: str
    artifact_id: str | None = None
    operation_id: str | None = None
    details: dict[str, Any] = field(default_factory=dict)
    evidence_refs: list[EvidenceRef] = field(default_factory=list)

    @classmethod
    def from_mapping(cls, raw: dict[str, Any]) -> "ValidationEvidenceRecord":
        required = ["validation_id", "run_id", "validator", "status"]
        missing = [key for key in required if not str(raw.get(key, "")).strip()]
        if missing:
            raise ValueError(f"validation evidence missing required fields: {', '.join(missing)}")
        status = str(raw["status"])
        if status not in VALIDATION_STATUSES:
            raise ValueError(f"invalid validation status: {status}")
        return cls(
            validation_id=str(raw["validation_id"]),
            run_id=str(raw["run_id"]),
            artifact_id=raw.get("artifact_id"),
            operation_id=raw.get("operation_id"),
            validator=str(raw["validator"]),
            status=status,
            details=dict(raw.get("details") or {}),
            evidence_refs=[EvidenceRef.from_mapping(item) for item in raw.get("evidence_refs", [])],
        )

    def schema_shape(self) -> dict[str, Any]:
        return {
            "validation_id": self.validation_id,
            "run_id": self.run_id,
            "artifact_id": self.artifact_id,
            "validator": self.validator,
            "status": self.status,
            "details": self.details,
            "evidence_refs": [item.uri for item in self.evidence_refs],
        }


class ValidationEvidenceStore:
    """SQLite-backed store for validation results and their proof evidence."""

    def __init__(self, conn: sqlite3.Connection, package_root: Path):
        self.conn = conn
        self.package_root = package_root

    def record(self, record: ValidationEvidenceRecord | dict[str, Any]) -> dict[str, Any]:
        item = record if isinstance(record, ValidationEvidenceRecord) else ValidationEvidenceRecord.from_mapping(record)
        self._require_run(item.run_id)
        if item.operation_id:
            self._require_operation(item.run_id, item.operation_id)
        if item.artifact_id:
            self._require_artifact(item.run_id, item.artifact_id)
        evidence_rows = self._upsert_evidence(item)
        evidence_uris = [row["uri"] for row in evidence_rows]
        details = {
            "details": item.details,
            "evidence_kinds": sorted({row["evidence_kind"] for row in evidence_rows}),
            "evidence_ids": [row["id"] for row in evidence_rows],
        }
        self.conn.execute(
            """
            INSERT INTO envctl_migration_validations
              (id, run_id, artifact_id, operation_id, validator, status, details_json, evidence_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
              run_id = excluded.run_id,
              artifact_id = excluded.artifact_id,
              operation_id = excluded.operation_id,
              validator = excluded.validator,
              status = excluded.status,
              details_json = excluded.details_json,
              evidence_json = excluded.evidence_json
            """,
            (
                item.validation_id,
                item.run_id,
                item.artifact_id,
                item.operation_id,
                item.validator,
                item.status,
                _json_dumps(details),
                _json_dumps(evidence_uris),
            ),
        )
        self.conn.commit()
        return {
            "validation_id": item.validation_id,
            "run_id": item.run_id,
            "artifact_id": item.artifact_id,
            "operation_id": item.operation_id,
            "validator": item.validator,
            "status": item.status,
            "evidence_ids": [row["id"] for row in evidence_rows],
            "evidence_uris": evidence_uris,
            "schema_shape": item.schema_shape(),
        }

    def scorecard(self, run_id: str) -> dict[str, Any]:
        self._require_run(run_id)
        row = self.conn.execute(
            """
            SELECT pass_count, fail_count, warn_count, blocked_count, unknown_count
            FROM envctl_migration_validation_scorecard
            WHERE run_id = ?
            """,
            (run_id,),
        ).fetchone()
        counts = dict(zip(["pass", "fail", "warn", "blocked", "unknown"], row or [0, 0, 0, 0, 0]))
        evidence_by_kind = {
            kind: count
            for kind, count in self.conn.execute(
                """
                SELECT evidence_kind, COUNT(*)
                FROM envctl_migration_evidence
                WHERE run_id = ?
                GROUP BY evidence_kind
                ORDER BY evidence_kind
                """,
                (run_id,),
            ).fetchall()
        }
        return {
            "run_id": run_id,
            "counts": counts,
            "evidence_by_kind": evidence_by_kind,
            "total": sum(counts.values()),
        }

    def _upsert_evidence(self, item: ValidationEvidenceRecord) -> list[dict[str, str | None]]:
        rows = []
        for ref in item.evidence_refs:
            self._validate_path_policy(ref.uri)
            path = _resolve_existing_path(self.package_root, ref.uri)
            actual_sha = _sha256_file(path) if path else None
            if ref.sha256 and actual_sha and ref.sha256 != actual_sha:
                raise ValueError(f"evidence hash mismatch for {ref.uri}")
            evidence_id = _stable_id("evidence", item.run_id, item.validation_id, ref.uri)
            metadata = {
                "validation_id": item.validation_id,
                "validator": item.validator,
                "status": item.status,
                "artifact_id": item.artifact_id,
                "details": item.details,
                **ref.metadata,
            }
            self.conn.execute(
                """
                INSERT INTO envctl_migration_evidence
                  (id, run_id, operation_id, uri, evidence_kind, sha256, redacted, metadata_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  operation_id = excluded.operation_id,
                  uri = excluded.uri,
                  evidence_kind = excluded.evidence_kind,
                  sha256 = excluded.sha256,
                  redacted = excluded.redacted,
                  metadata_json = excluded.metadata_json
                """,
                (
                    evidence_id,
                    item.run_id,
                    item.operation_id,
                    ref.uri,
                    ref.evidence_kind,
                    ref.sha256 or actual_sha,
                    1 if ref.redacted else 0,
                    _json_dumps(metadata),
                ),
            )
            rows.append(
                {
                    "id": evidence_id,
                    "uri": ref.uri,
                    "evidence_kind": ref.evidence_kind,
                    "sha256": ref.sha256 or actual_sha,
                }
            )
        return rows

    def _validate_path_policy(self, relpath: str) -> None:
        normalized = relpath.replace("\\", "/")
        path = Path(normalized)
        if path.is_absolute() or ".." in path.parts:
            raise ValueError(f"validation evidence path must be package-relative: {relpath}")
        for pattern in BLOCKED_PATH_PATTERNS:
            if fnmatch.fnmatch(normalized, pattern) or fnmatch.fnmatch(f"root/{normalized}", pattern):
                raise ValueError(f"validation evidence path is blocked by policy: {relpath}")

    def _require_run(self, run_id: str) -> None:
        row = self.conn.execute("SELECT 1 FROM envctl_migration_runs WHERE id = ?", (run_id,)).fetchone()
        if row is None:
            raise ValueError(f"unknown run_id: {run_id}")

    def _require_operation(self, run_id: str, operation_id: str) -> None:
        row = self.conn.execute(
            "SELECT 1 FROM envctl_migration_operations WHERE run_id = ? AND id = ?",
            (run_id, operation_id),
        ).fetchone()
        if row is None:
            raise ValueError(f"operation does not belong to run: {operation_id}")

    def _require_artifact(self, run_id: str, artifact_id: str) -> None:
        row = self.conn.execute(
            "SELECT 1 FROM envctl_migration_artifacts WHERE run_id = ? AND artifact_id = ?",
            (run_id, artifact_id),
        ).fetchone()
        if row is None:
            raise ValueError(f"artifact does not belong to run: {artifact_id}")


def fetch_validation(conn: sqlite3.Connection, validation_id: str) -> dict[str, Any]:
    row = conn.execute(
        """
        SELECT id, run_id, artifact_id, operation_id, validator, status, details_json, evidence_json
        FROM envctl_migration_validations
        WHERE id = ?
        """,
        (validation_id,),
    ).fetchone()
    if row is None:
        raise KeyError(f"validation not found: {validation_id}")
    columns = [
        "id",
        "run_id",
        "artifact_id",
        "operation_id",
        "validator",
        "status",
        "details_json",
        "evidence_json",
    ]
    out = dict(zip(columns, row))
    out["details"] = json.loads(out.pop("details_json") or "{}")
    out["evidence"] = json.loads(out.pop("evidence_json") or "[]")
    return out
