//! redb plumbing: one table per DDL entity (JSON values keyed by id), plus the
//! secondary index tables that stand in for the SQL indexes (sql/002). Key charset
//! is `[a-z0-9._/@-]` with `#` as the composite separator; `~` (0x7E) upper-bounds
//! every legal key, which makes prefix scans a simple range.

use super::{MigrationDb, MigrationDbError, Result};
use redb::{ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub const TARGETS: TableDefinition<&str, &[u8]> = TableDefinition::new("targets");
pub const PACKAGES: TableDefinition<&str, &[u8]> = TableDefinition::new("packages");
pub const CONTRACTS: TableDefinition<&str, &[u8]> = TableDefinition::new("artifact_contracts");
pub const RECIPES: TableDefinition<&str, &[u8]> = TableDefinition::new("recipes");
pub const RUNS: TableDefinition<&str, &[u8]> = TableDefinition::new("runs");
pub const OPERATIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("operations");
/// Events keyed `run_id#seq012` — natural per-run ordering, range-scannable.
pub const RUN_EVENTS: TableDefinition<&str, &[u8]> = TableDefinition::new("run_events");
pub const EVIDENCE: TableDefinition<&str, &[u8]> = TableDefinition::new("evidence");
pub const ARTIFACTS: TableDefinition<&str, &[u8]> = TableDefinition::new("artifacts");
pub const GRAPH_EDGES: TableDefinition<&str, &[u8]> = TableDefinition::new("graph_edges");
pub const APPROVALS: TableDefinition<&str, &[u8]> = TableDefinition::new("approvals");
pub const VALIDATIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("validations");
pub const CHECKPOINTS: TableDefinition<&str, &[u8]> = TableDefinition::new("checkpoints");
pub const ROLLBACKS: TableDefinition<&str, &[u8]> = TableDefinition::new("rollbacks");
pub const AGENT_SESSIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("agent_sessions");
pub const PLUGIN_SESSIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("plugin_sessions");

/// Secondary indexes (`run_id#child_id` -> child id) — the sql/002 index equivalents.
pub const IDX_OPS_BY_RUN: TableDefinition<&str, &str> = TableDefinition::new("idx_ops_by_run");
pub const IDX_EVIDENCE_BY_RUN: TableDefinition<&str, &str> =
    TableDefinition::new("idx_evidence_by_run");
pub const IDX_ARTIFACTS_BY_RUN: TableDefinition<&str, &str> =
    TableDefinition::new("idx_artifacts_by_run");
pub const IDX_APPROVALS_BY_RUN: TableDefinition<&str, &str> =
    TableDefinition::new("idx_approvals_by_run");
pub const IDX_VALIDATIONS_BY_RUN: TableDefinition<&str, &str> =
    TableDefinition::new("idx_validations_by_run");
pub const IDX_EDGES_BY_RUN: TableDefinition<&str, &str> = TableDefinition::new("idx_edges_by_run");
pub const IDX_CHECKPOINTS_BY_RUN: TableDefinition<&str, &str> =
    TableDefinition::new("idx_checkpoints_by_run");
pub const IDX_ROLLBACKS_BY_RUN: TableDefinition<&str, &str> =
    TableDefinition::new("idx_rollbacks_by_run");
/// UNIQUE(run_id, idempotency_key): `run_id#idem_key` -> operation id.
pub const IDX_OP_IDEMPOTENCY: TableDefinition<&str, &str> =
    TableDefinition::new("idx_op_idempotency");
/// UNIQUE(target_id) natural-key index -> row id.
pub const IDX_TARGET_NATURAL: TableDefinition<&str, &str> =
    TableDefinition::new("idx_target_natural");
/// UNIQUE(contract_name, contract_version) -> row id; same for recipes.
pub const IDX_CONTRACT_NATURAL: TableDefinition<&str, &str> =
    TableDefinition::new("idx_contract_natural");
pub const IDX_RECIPE_NATURAL: TableDefinition<&str, &str> =
    TableDefinition::new("idx_recipe_natural");
/// Monotonic id/seq counters: kind -> next u64.
pub const COUNTERS: TableDefinition<&str, u64> = TableDefinition::new("counters");

fn store_err<E: std::fmt::Display>(e: E) -> MigrationDbError {
    MigrationDbError::Store(e.to_string())
}

/// Create every table once at open so readers never race existence.
pub fn create_all_tables(mdb: &MigrationDb) -> Result<()> {
    let wtx = mdb.db.begin_write().map_err(store_err)?;
    {
        wtx.open_table(TARGETS).map_err(store_err)?;
        wtx.open_table(PACKAGES).map_err(store_err)?;
        wtx.open_table(CONTRACTS).map_err(store_err)?;
        wtx.open_table(RECIPES).map_err(store_err)?;
        wtx.open_table(RUNS).map_err(store_err)?;
        wtx.open_table(OPERATIONS).map_err(store_err)?;
        wtx.open_table(RUN_EVENTS).map_err(store_err)?;
        wtx.open_table(EVIDENCE).map_err(store_err)?;
        wtx.open_table(ARTIFACTS).map_err(store_err)?;
        wtx.open_table(GRAPH_EDGES).map_err(store_err)?;
        wtx.open_table(APPROVALS).map_err(store_err)?;
        wtx.open_table(VALIDATIONS).map_err(store_err)?;
        wtx.open_table(CHECKPOINTS).map_err(store_err)?;
        wtx.open_table(ROLLBACKS).map_err(store_err)?;
        wtx.open_table(AGENT_SESSIONS).map_err(store_err)?;
        wtx.open_table(PLUGIN_SESSIONS).map_err(store_err)?;
        wtx.open_table(IDX_OPS_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_EVIDENCE_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_ARTIFACTS_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_APPROVALS_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_VALIDATIONS_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_EDGES_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_CHECKPOINTS_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_ROLLBACKS_BY_RUN).map_err(store_err)?;
        wtx.open_table(IDX_OP_IDEMPOTENCY).map_err(store_err)?;
        wtx.open_table(IDX_TARGET_NATURAL).map_err(store_err)?;
        wtx.open_table(IDX_CONTRACT_NATURAL).map_err(store_err)?;
        wtx.open_table(IDX_RECIPE_NATURAL).map_err(store_err)?;
        wtx.open_table(COUNTERS).map_err(store_err)?;
    }
    wtx.commit().map_err(store_err)
}

impl MigrationDb {
    /// Serialize + insert. `unique = true` refuses overwrite (DDL PRIMARY KEY / UNIQUE).
    pub(crate) fn put<T: Serialize>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        key: &str,
        value: &T,
        unique: bool,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(value)?;
        let wtx = self.db.begin_write().map_err(store_err)?;
        {
            let mut t = wtx.open_table(table).map_err(store_err)?;
            if unique {
                let exists = t.get(key).map_err(store_err)?.is_some();
                if exists {
                    return Err(MigrationDbError::Conflict(format!(
                        "key already exists: {key}"
                    )));
                }
            }
            t.insert(key, bytes.as_slice()).map_err(store_err)?;
        }
        wtx.commit().map_err(store_err)
    }

    pub(crate) fn get<T: DeserializeOwned>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Result<Option<T>> {
        let rtx = self.db.begin_read().map_err(store_err)?;
        let t = rtx.open_table(table).map_err(store_err)?;
        match t.get(key).map_err(store_err)? {
            Some(guard) => Ok(Some(serde_json::from_slice(guard.value())?)),
            None => Ok(None),
        }
    }

    pub(crate) fn must_get<T: DeserializeOwned>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        key: &str,
        what: &str,
    ) -> Result<T> {
        self.get(table, key)?
            .ok_or_else(|| MigrationDbError::NotFound(format!("{what}: {key}")))
    }

    /// List all values, optionally constrained to keys starting with `prefix`.
    pub(crate) fn list<T: DeserializeOwned>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        prefix: Option<&str>,
    ) -> Result<Vec<T>> {
        let rtx = self.db.begin_read().map_err(store_err)?;
        let t = rtx.open_table(table).map_err(store_err)?;
        let mut out = Vec::new();
        match prefix {
            Some(p) => {
                let end = format!("{p}~");
                for item in t.range(p..end.as_str()).map_err(store_err)? {
                    let (_, v) = item.map_err(store_err)?;
                    out.push(serde_json::from_slice(v.value())?);
                }
            }
            None => {
                for item in t.iter().map_err(store_err)? {
                    let (_, v) = item.map_err(store_err)?;
                    out.push(serde_json::from_slice(v.value())?);
                }
            }
        }
        Ok(out)
    }

    /// Insert into a `&str -> &str` index; refuses duplicates when `unique`.
    pub(crate) fn index_put(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
        value: &str,
        unique: bool,
    ) -> Result<()> {
        let wtx = self.db.begin_write().map_err(store_err)?;
        {
            let mut t = wtx.open_table(table).map_err(store_err)?;
            if unique {
                let exists = t.get(key).map_err(store_err)?.is_some();
                if exists {
                    return Err(MigrationDbError::Conflict(format!(
                        "unique index violation: {key}"
                    )));
                }
            }
            t.insert(key, value).map_err(store_err)?;
        }
        wtx.commit().map_err(store_err)
    }

    pub(crate) fn index_get(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Option<String>> {
        let rtx = self.db.begin_read().map_err(store_err)?;
        let t = rtx.open_table(table).map_err(store_err)?;
        Ok(t.get(key)
            .map_err(store_err)?
            .map(|g| g.value().to_string()))
    }

    /// Child ids for a run from a `run_id#child` index.
    pub(crate) fn index_children(
        &self,
        table: TableDefinition<&str, &str>,
        run_id: &str,
    ) -> Result<Vec<String>> {
        let rtx = self.db.begin_read().map_err(store_err)?;
        let t = rtx.open_table(table).map_err(store_err)?;
        let start = format!("{run_id}#");
        let end = format!("{run_id}#~");
        let mut out = Vec::new();
        for item in t.range(start.as_str()..end.as_str()).map_err(store_err)? {
            let (_, v) = item.map_err(store_err)?;
            out.push(v.value().to_string());
        }
        Ok(out)
    }

    /// Next value of a named monotonic counter (starts at 1).
    pub(crate) fn next_counter(&self, kind: &str) -> Result<u64> {
        let wtx = self.db.begin_write().map_err(store_err)?;
        let next = {
            let mut t = wtx.open_table(COUNTERS).map_err(store_err)?;
            let current = t
                .get(kind)
                .map_err(store_err)?
                .map(|g| g.value())
                .unwrap_or(0);
            let next = current + 1;
            t.insert(kind, next).map_err(store_err)?;
            next
        };
        wtx.commit().map_err(store_err)?;
        Ok(next)
    }

    /// Generate the next display id for an entity kind, e.g. `run-000007`.
    pub(crate) fn next_id(&self, kind: &str) -> Result<String> {
        Ok(format!("{kind}-{:06}", self.next_counter(kind)?))
    }
}

/// Composite key for the per-run event table: zero-padded so lexical = numeric order.
pub fn event_key(run_id: &str, seq: u64) -> String {
    format!("{run_id}#{seq:012}")
}

/// Composite key for run-scoped child indexes.
pub fn child_key(run_id: &str, child_id: &str) -> String {
    format!("{run_id}#{child_id}")
}
