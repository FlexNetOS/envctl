from __future__ import annotations

import hashlib
import fnmatch
import json
import re
import sqlite3
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


VALID_ARTIFACT_STATUSES = {"complete", "partial", "unknown", "blocked"}
VALID_VALIDATION_STATUSES = {"pass", "fail", "warn", "blocked", "unknown"}
BLOCKED_PATH_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")


def _stable_id(prefix: str, *parts: str) -> str:
    digest = hashlib.sha256("\0".join(parts).encode("utf-8")).hexdigest()[:24]
    safe_prefix = re.sub(r"[^a-zA-Z0-9_-]+", "-", prefix).strip("-")
    return f"{safe_prefix}-{digest}"


def _json_dumps(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


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
class ValidationLink:
    validator: str
    status: str
    details: dict[str, Any] = field(default_factory=dict)
    evidence_refs: list[str] = field(default_factory=list)
    operation_id: str | None = None

    @classmethod
    def from_mapping(cls, raw: dict[str, Any]) -> "ValidationLink":
        status = str(raw.get("status", "unknown"))
        if status not in VALID_VALIDATION_STATUSES:
            raise ValueError(f"invalid validation status: {status}")
        validator = str(raw.get("validator", "")).strip()
        if not validator:
            raise ValueError("validation link requires validator")
        return cls(
            validator=validator,
            status=status,
            details=dict(raw.get("details") or {}),
            evidence_refs=[str(item) for item in raw.get("evidence_refs", [])],
            operation_id=raw.get("operation_id"),
        )


@dataclass(frozen=True)
class ArtifactRecord:
    artifact_id: str
    run_id: str
    title: str
    status: str
    artifact_type: str | None = None
    path: str | None = None
    content_hash: str | None = None
    producer_operation_id: str | None = None
    contract_id: str | None = None
    provenance: dict[str, Any] = field(default_factory=dict)
    evidence_refs: list[str] = field(default_factory=list)
    links: list[dict[str, str]] = field(default_factory=list)
    validations: list[ValidationLink] = field(default_factory=list)

    @classmethod
    def from_mapping(cls, raw: dict[str, Any]) -> "ArtifactRecord":
        required = ["artifact_id", "run_id", "title", "status"]
        missing = [key for key in required if not str(raw.get(key, "")).strip()]
        if missing:
            raise ValueError(f"artifact record missing required fields: {', '.join(missing)}")
        status = str(raw["status"])
        if status not in VALID_ARTIFACT_STATUSES:
            raise ValueError(f"invalid artifact status: {status}")
        links = []
        for link in raw.get("links", []):
            if not isinstance(link, dict) or not link.get("to") or not link.get("type"):
                raise ValueError("artifact links must contain to and type")
            links.append({"to": str(link["to"]), "type": str(link["type"])})
        return cls(
            artifact_id=str(raw["artifact_id"]),
            run_id=str(raw["run_id"]),
            title=str(raw["title"]),
            status=status,
            artifact_type=raw.get("artifact_type"),
            path=raw.get("path"),
            content_hash=raw.get("content_hash"),
            producer_operation_id=raw.get("producer_operation_id"),
            contract_id=raw.get("contract_id"),
            provenance=dict(raw.get("provenance") or {}),
            evidence_refs=[str(item) for item in raw.get("evidence_refs", [])],
            links=links,
            validations=[ValidationLink.from_mapping(item) for item in raw.get("validations", [])],
        )


class ArtifactRegistry:
    """SQLite-backed registry for migration artifacts and their validation links."""

    def __init__(self, conn: sqlite3.Connection, package_root: Path):
        self.conn = conn
        self.package_root = package_root

    def register(self, record: ArtifactRecord | dict[str, Any]) -> dict[str, Any]:
        item = record if isinstance(record, ArtifactRecord) else ArtifactRecord.from_mapping(record)
        content_hash = self._validated_content_hash(item)
        evidence_rows = self._upsert_evidence(item)
        validation_rows = self._upsert_validations(item)
        graph_edges = self._upsert_graph_edges(item)
        artifact_db_id = _stable_id("artifact", item.run_id, item.artifact_id)
        evidence_payload = {
            "evidence_refs": item.evidence_refs,
            "evidence_ids": [row["id"] for row in evidence_rows],
            "producer_operation_id": item.producer_operation_id,
            "contract_id": item.contract_id,
            "provenance": item.provenance,
            "validation_ids": [row["id"] for row in validation_rows],
        }
        links_payload = {
            "links": item.links,
            "contract_id": item.contract_id,
            "graph_edge_ids": [row["id"] for row in graph_edges],
        }
        self.conn.execute(
            """
            INSERT INTO envctl_migration_artifacts
              (id, run_id, artifact_id, title, artifact_type, status, path,
               content_hash, generated_by_operation_id, evidence_json, links_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(run_id, artifact_id) DO UPDATE SET
              title = excluded.title,
              artifact_type = excluded.artifact_type,
              status = excluded.status,
              path = excluded.path,
              content_hash = excluded.content_hash,
              generated_by_operation_id = excluded.generated_by_operation_id,
              evidence_json = excluded.evidence_json,
              links_json = excluded.links_json,
              updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ','now')
            """,
            (
                artifact_db_id,
                item.run_id,
                item.artifact_id,
                item.title,
                item.artifact_type,
                item.status,
                item.path,
                content_hash,
                item.producer_operation_id,
                _json_dumps(evidence_payload),
                _json_dumps(links_payload),
            ),
        )
        self.conn.commit()
        return {
            "artifact_row_id": artifact_db_id,
            "artifact_id": item.artifact_id,
            "run_id": item.run_id,
            "path": item.path,
            "content_hash": content_hash,
            "producer_operation_id": item.producer_operation_id,
            "contract_id": item.contract_id,
            "evidence_ids": [row["id"] for row in evidence_rows],
            "graph_edge_ids": [row["id"] for row in graph_edges],
            "validation_ids": [row["id"] for row in validation_rows],
        }

    def _validated_content_hash(self, item: ArtifactRecord) -> str | None:
        self._require_run(item.run_id)
        if item.producer_operation_id:
            self._require_operation(item.run_id, item.producer_operation_id)
        if item.contract_id:
            self._require_contract_for_run(item.run_id, item.contract_id)
        self._validate_path_policy(item.path)
        for ref in item.evidence_refs:
            self._validate_path_policy(ref)
        actual = self._hash_for_path(item.path)
        if item.content_hash and actual and item.content_hash != actual:
            raise ValueError(f"artifact content hash mismatch for {item.path}")
        return item.content_hash or actual

    def _hash_for_path(self, relpath: str | None) -> str | None:
        path = _resolve_existing_path(self.package_root, relpath)
        return _sha256_file(path) if path else None

    def _validate_path_policy(self, relpath: str | None) -> None:
        if not relpath:
            return
        normalized = relpath.replace("\\", "/")
        path = Path(normalized)
        if path.is_absolute() or ".." in path.parts:
            raise ValueError(f"artifact registry path must be package-relative: {relpath}")
        for pattern in BLOCKED_PATH_PATTERNS:
            if fnmatch.fnmatch(normalized, pattern) or fnmatch.fnmatch(f"root/{normalized}", pattern):
                raise ValueError(f"artifact registry path is blocked by policy: {relpath}")

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

    def _require_contract_for_run(self, run_id: str, contract_id: str) -> None:
        row = self.conn.execute(
            "SELECT artifact_contract_id FROM envctl_migration_runs WHERE id = ?",
            (run_id,),
        ).fetchone()
        if row is None:
            raise ValueError(f"unknown run_id: {run_id}")
        if row[0] != contract_id:
            raise ValueError(f"contract id does not match run contract: {contract_id}")
        exists = self.conn.execute(
            "SELECT 1 FROM envctl_migration_artifact_contracts WHERE id = ?",
            (contract_id,),
        ).fetchone()
        if exists is None:
            raise ValueError(f"unknown artifact contract: {contract_id}")

    def _upsert_evidence(self, item: ArtifactRecord) -> list[dict[str, str | None]]:
        rows = []
        for ref in item.evidence_refs:
            path = _resolve_existing_path(self.package_root, ref)
            evidence_id = _stable_id("evidence", item.run_id, item.artifact_id, ref)
            sha = _sha256_file(path) if path else None
            metadata = {
                "artifact_id": item.artifact_id,
                "artifact_path": item.path,
                "contract_id": item.contract_id,
                "provenance": item.provenance,
            }
            self.conn.execute(
                """
                INSERT INTO envctl_migration_evidence
                  (id, run_id, operation_id, uri, evidence_kind, sha256, redacted, metadata_json)
                VALUES (?, ?, ?, ?, ?, ?, 0, ?)
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
                    item.producer_operation_id,
                    ref,
                    "artifact_registry_link",
                    sha,
                    _json_dumps(metadata),
                ),
            )
            rows.append({"id": evidence_id, "uri": ref, "sha256": sha})
        return rows

    def _upsert_graph_edges(self, item: ArtifactRecord) -> list[dict[str, str]]:
        links = list(item.links)
        if item.contract_id:
            links.append({"to": item.contract_id, "type": "satisfies_contract"})
        rows = []
        evidence_json = _json_dumps(item.evidence_refs)
        for link in links:
            edge_id = _stable_id("edge", item.run_id, item.artifact_id, link["to"], link["type"])
            self.conn.execute(
                """
                INSERT INTO envctl_migration_graph_edges
                  (id, run_id, from_node, to_node, edge_type, source_artifact_id, confidence, evidence_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  from_node = excluded.from_node,
                  to_node = excluded.to_node,
                  edge_type = excluded.edge_type,
                  source_artifact_id = excluded.source_artifact_id,
                  confidence = excluded.confidence,
                  evidence_json = excluded.evidence_json
                """,
                (
                    edge_id,
                    item.run_id,
                    f"artifact:{item.artifact_id}",
                    link["to"],
                    link["type"],
                    item.artifact_id,
                    "high",
                    evidence_json,
                ),
            )
            rows.append({"id": edge_id, "to": link["to"], "type": link["type"]})
        return rows

    def _upsert_validations(self, item: ArtifactRecord) -> list[dict[str, str]]:
        rows = []
        for validation in item.validations:
            operation_id = validation.operation_id or item.producer_operation_id
            validation_id = _stable_id("validation", item.run_id, item.artifact_id, validation.validator)
            details = {
                "artifact_path": item.path,
                "contract_id": item.contract_id,
                "details": validation.details,
            }
            self.conn.execute(
                """
                INSERT INTO envctl_migration_validations
                  (id, run_id, artifact_id, operation_id, validator, status, details_json, evidence_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  artifact_id = excluded.artifact_id,
                  operation_id = excluded.operation_id,
                  validator = excluded.validator,
                  status = excluded.status,
                  details_json = excluded.details_json,
                  evidence_json = excluded.evidence_json
                """,
                (
                    validation_id,
                    item.run_id,
                    item.artifact_id,
                    operation_id,
                    validation.validator,
                    validation.status,
                    _json_dumps(details),
                    _json_dumps(validation.evidence_refs or item.evidence_refs),
                ),
            )
            rows.append({"id": validation_id, "validator": validation.validator, "status": validation.status})
        return rows


def fetch_artifact(conn: sqlite3.Connection, run_id: str, artifact_id: str) -> dict[str, Any]:
    row = conn.execute(
        """
        SELECT id, run_id, artifact_id, title, artifact_type, status, path, content_hash,
               generated_by_operation_id, evidence_json, links_json
        FROM envctl_migration_artifacts
        WHERE run_id = ? AND artifact_id = ?
        """,
        (run_id, artifact_id),
    ).fetchone()
    if row is None:
        raise KeyError(f"artifact not found: {run_id}/{artifact_id}")
    columns = [
        "id",
        "run_id",
        "artifact_id",
        "title",
        "artifact_type",
        "status",
        "path",
        "content_hash",
        "generated_by_operation_id",
        "evidence_json",
        "links_json",
    ]
    out = dict(zip(columns, row))
    out["evidence"] = json.loads(out.pop("evidence_json") or "{}")
    out["links"] = json.loads(out.pop("links_json") or "{}")
    return out
