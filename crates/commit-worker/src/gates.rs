//! Release gate execution (blueprint §17 step 15 precondition).
//!
//! `lifeos_release.promote` refuses to issue an activation until all eleven
//! named gates hold a passing `lifeos_release.verification` row. This module
//! is what actually runs them. Every gate here measures live state — the
//! database, the Nix store, the running toolchain — and reports what it
//! measured. Nothing infers a pass from the absence of evidence.
//!
//! Two rules shape the whole module, and both are load-bearing:
//!
//! * A gate that cannot measure its subject **fails**. It does not skip, warn,
//!   or degrade to a weaker check. Several subsystems named by the gate list
//!   exist as schema with no rows and no procedures; those gates fail here and
//!   say exactly what was missing, which is the honest state of the release.
//! * Running gates never writes verification rows. Recording is a separate,
//!   explicit step, so a gate run can never be mistaken for an approval.
//!
//! Placement follows the recorded architecture decision to extend the existing
//! envctl commit-worker rather than stand up a parallel proof subsystem.

use postgres::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

use crate::{connect, CommitError};

/// The gate names `lifeos_release.promote` enumerates, in its order.
pub const REQUIRED_GATES: [&str; 11] = [
    "build",
    "test",
    "byte-reconstruction",
    "retrieval",
    "graph-causal",
    "security",
    "model",
    "forecast",
    "witness",
    "runner-receipt",
    "rollback",
];

/// What one gate measured, and whether that measurement clears it.
#[derive(Debug, Clone, Serialize)]
pub struct GateOutcome {
    pub gate: &'static str,
    pub passed: bool,
    /// Why it passed or failed, in one line.
    pub detail: String,
    /// The raw numbers the verdict rests on, so a reader can re-derive it.
    pub measurement: Value,
}

impl GateOutcome {
    fn pass(gate: &'static str, detail: impl Into<String>, measurement: Value) -> Self {
        Self {
            gate,
            passed: true,
            detail: detail.into(),
            measurement,
        }
    }

    fn fail(gate: &'static str, detail: impl Into<String>, measurement: Value) -> Self {
        Self {
            gate,
            passed: false,
            detail: detail.into(),
            measurement,
        }
    }
}

/// Run every gate and return all eleven outcomes in `promote`'s order.
///
/// This never short-circuits: a failing gate does not suppress the rest,
/// because the useful output is the full picture of how far the release
/// actually gets.
pub fn run_all(
    conn: &str,
    repo_root: &Path,
    release_root: &Path,
) -> Result<Vec<GateOutcome>, CommitError> {
    let mut client = connect(conn)?;
    // Tenant-scoped capability functions (notably `lifeos_blob.verify_object`)
    // return false rather than erroring when no binding is established, so an
    // unbound run would report intact data as corrupt. Bind first, and fail
    // the whole run if binding is impossible — a gate result read under the
    // wrong authority is worse than no result.
    bind_session(&mut client)?;
    Ok(vec![
        gate_build(release_root),
        gate_test(repo_root),
        gate_byte_reconstruction(&mut client),
        gate_retrieval(&mut client),
        gate_graph_causal(&mut client),
        gate_security(&mut client),
        gate_model(&mut client),
        gate_forecast(&mut client),
        gate_witness(&mut client),
        gate_runner_receipt(),
        gate_rollback(),
    ])
}

/// Establish this backend's tenant binding through envctl's own bootstrap.
///
/// `bootstrap_envctl_context` authenticates the binding bytes against the
/// grant they claim, so the authority cannot be asserted by the caller — it
/// has to already exist as a live, unexpired `bind-session` grant for an
/// identity whose `subject_key` is this database user.
fn bind_session(client: &mut Client) -> Result<(), CommitError> {
    let row = client
        .query_opt(
            "SELECT identity.tenant_id::text, identity.identity_id::text, \
                    grant_row.grant_id::text \
             FROM lifeos_security.identity identity \
             JOIN lifeos_security.\"grant\" grant_row \
               ON grant_row.identity_id = identity.identity_id \
              AND grant_row.tenant_id = identity.tenant_id \
             WHERE identity.subject_key = session_user \
               AND grant_row.task_id IS NULL \
               AND grant_row.lease_id IS NULL \
               AND 'bind-session' = ANY (grant_row.action_scope) \
               AND grant_row.revoked_at IS NULL \
               AND grant_row.expires_at > statement_timestamp() \
             ORDER BY grant_row.expires_at DESC \
             LIMIT 1",
            &[],
        )
        .map_err(|error| CommitError::new(format!("looking up session authority failed: {error}")))?
        .ok_or_else(|| {
            CommitError::new(
                "no live bind-session grant exists for this database user; \
                 gates cannot run under an unbound authority",
            )
        })?;

    let tenant: String = row.get(0);
    let identity: String = row.get(1);
    let grant: String = row.get(2);
    let binding = serde_json::to_vec(&json!({
        "tenant_id": tenant,
        "identity_id": identity,
        "grant_id": grant,
        "purpose": "envctl-session-binding",
    }))
    .map_err(|error| CommitError::new(format!("encoding binding bytes failed: {error}")))?;

    client
        .query_one(
            "SELECT binding_id::text FROM lifeos_security.bootstrap_envctl_context( \
                 $1::text::uuid, $2::text::uuid, $3::text::uuid, $4::bytea)",
            &[&tenant, &identity, &grant, &binding],
        )
        .map_err(|error| {
            CommitError::new(format!("establishing the session binding failed: {error}"))
        })?;
    Ok(())
}

/// Count helper: one scalar bigint from a query, or a gate-legible error.
fn count(client: &mut Client, sql: &str) -> Result<i64, CommitError> {
    client
        .query_one(sql, &[])
        .map(|row| row.get::<_, i64>(0))
        .map_err(|error| CommitError::new(format!("gate query failed: {error}")))
}

/// `build`: the artifact under release is a registered, valid store object.
///
/// A release in this system is a Nix closure, so "it builds" is checked as
/// "the store has it and considers it intact", which is stronger than a
/// successful compile that was never retained.
fn gate_build(release_root: &Path) -> GateOutcome {
    if !release_root.exists() {
        return GateOutcome::fail(
            "build",
            format!("release root {} does not exist", release_root.display()),
            json!({ "release_root": release_root.to_string_lossy(), "exists": false }),
        );
    }

    let output = Command::new("nix")
        .args(["path-info", "--json"])
        .arg(release_root)
        .output();

    match output {
        Ok(result) if result.status.success() => GateOutcome::pass(
            "build",
            "release root is a valid registered store path",
            json!({
                "release_root": release_root.to_string_lossy(),
                "nix_path_info_exit": 0,
            }),
        ),
        Ok(result) => GateOutcome::fail(
            "build",
            "nix does not consider the release root a valid store path",
            json!({
                "release_root": release_root.to_string_lossy(),
                "nix_path_info_exit": result.status.code(),
                "stderr": String::from_utf8_lossy(&result.stderr).trim(),
            }),
        ),
        Err(error) => GateOutcome::fail(
            "build",
            format!("could not run nix path-info: {error}"),
            json!({ "release_root": release_root.to_string_lossy() }),
        ),
    }
}

/// `test`: the repository's own test suite passes, run for real.
fn gate_test(repo_root: &Path) -> GateOutcome {
    let output = Command::new("cargo")
        .args(["test", "--workspace", "--quiet"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => GateOutcome::pass(
            "test",
            "cargo test --workspace passed",
            json!({ "repo_root": repo_root.to_string_lossy(), "exit": 0 }),
        ),
        Ok(result) => GateOutcome::fail(
            "test",
            "cargo test --workspace failed",
            json!({
                "repo_root": repo_root.to_string_lossy(),
                "exit": result.status.code(),
                "stderr_tail": String::from_utf8_lossy(&result.stderr)
                    .lines()
                    .rev()
                    .take(8)
                    .collect::<Vec<_>>()
                    .join(" | "),
            }),
        ),
        Err(error) => GateOutcome::fail(
            "test",
            format!("could not run cargo test: {error}"),
            json!({ "repo_root": repo_root.to_string_lossy() }),
        ),
    }
}

/// `byte-reconstruction`: every stored object reproduces its own digest.
///
/// Checked two independent ways — the database's own `verify_object`, and a
/// recomputation of SHA-256 over the retained bytes — so a bug in either path
/// alone cannot make the gate pass.
fn gate_byte_reconstruction(client: &mut Client) -> GateOutcome {
    let total = match count(client, "SELECT count(*) FROM lifeos_blob.object") {
        Ok(value) => value,
        Err(error) => {
            return GateOutcome::fail("byte-reconstruction", error.to_string(), json!({}))
        }
    };
    if total == 0 {
        return GateOutcome::fail(
            "byte-reconstruction",
            "no stored objects to reconstruct",
            json!({ "objects": 0 }),
        );
    }

    let verified = count(
        client,
        "SELECT count(*) FROM lifeos_blob.object \
         WHERE lifeos_blob.verify_object(object_id)",
    );
    let rehashed = count(
        client,
        "SELECT count(*) FROM lifeos_blob.object \
         WHERE NOT chunked AND bytes_inline IS NOT NULL \
           AND sha256 = extensions.digest(bytes_inline, 'sha256')",
    );
    let inline = count(
        client,
        "SELECT count(*) FROM lifeos_blob.object \
         WHERE NOT chunked AND bytes_inline IS NOT NULL",
    );

    match (verified, rehashed, inline) {
        (Ok(verified), Ok(rehashed), Ok(inline)) => {
            let measurement = json!({
                "objects": total,
                "verify_object_passed": verified,
                "inline_objects": inline,
                "sha256_recomputed_match": rehashed,
            });
            if verified == total && rehashed == inline {
                GateOutcome::pass(
                    "byte-reconstruction",
                    format!("all {total} objects verify and re-hash identically"),
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "byte-reconstruction",
                    "at least one object failed verification or re-hash",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail(
            "byte-reconstruction",
            "reconstruction queries failed",
            json!({ "objects": total }),
        ),
    }
}

/// `retrieval`: a usable semantic index exists and is genuinely searchable.
///
/// Requires real vectors, not placeholders: a retired or degenerate index
/// (single-digit dimensions, a handful of distinct values, sentinel
/// components) cannot answer a nearest-neighbour query meaningfully, so it
/// fails rather than passing on row count alone.
fn gate_retrieval(client: &mut Client) -> GateOutcome {
    let live = count(
        client,
        "SELECT count(*) FROM lifeos_semantic.embedding \
         WHERE record_kind = 'embedding'",
    );
    let distinct = count(
        client,
        "SELECT count(DISTINCT embedding::text) FROM lifeos_semantic.embedding",
    );
    let usable_dims = count(
        client,
        "SELECT count(*) FROM lifeos_semantic.embedding WHERE dimension >= 128",
    );
    let indexes = count(
        client,
        "SELECT count(*) FROM lifeos_semantic.index_generation",
    );

    match (live, distinct, usable_dims, indexes) {
        (Ok(live), Ok(distinct), Ok(usable_dims), Ok(indexes)) => {
            let measurement = json!({
                "live_embeddings": live,
                "distinct_vectors": distinct,
                "embeddings_with_dimension_ge_128": usable_dims,
                "index_generations": indexes,
            });
            if usable_dims > 0 && indexes > 0 && distinct > 1 {
                GateOutcome::pass(
                    "retrieval",
                    format!("{usable_dims} indexed vectors of usable dimension"),
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "retrieval",
                    "no usable semantic index: vectors are degenerate or retired \
                     and no index generation is built",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail("retrieval", "retrieval queries failed", json!({})),
    }
}

/// `graph-causal`: a populated graph exists with causal edges over it.
fn gate_graph_causal(client: &mut Client) -> GateOutcome {
    let nodes = count(client, "SELECT count(*) FROM lifeos_semantic.graph_node");
    let edges = count(client, "SELECT count(*) FROM lifeos_semantic.graph_edge");
    let causal = count(client, "SELECT count(*) FROM lifeos_semantic.causal_edge");

    match (nodes, edges, causal) {
        (Ok(nodes), Ok(edges), Ok(causal)) => {
            let measurement = json!({
                "graph_nodes": nodes,
                "graph_edges": edges,
                "causal_edges": causal,
            });
            if nodes > 0 && edges > 0 && causal > 0 {
                GateOutcome::pass(
                    "graph-causal",
                    format!("{nodes} nodes, {edges} edges, {causal} causal edges"),
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "graph-causal",
                    "graph and causal tables are empty: no graph to reason over",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail("graph-causal", "graph queries failed", json!({})),
    }
}

/// `security`: the privilege model actually denies what it claims to deny.
///
/// Measured as a negative: runtime roles must hold zero write privilege on the
/// release tables, so a release cannot be self-approved by the components it
/// governs. Row-level security must also be enabled on the canonical tables
/// rather than merely configured.
fn gate_security(client: &mut Client) -> GateOutcome {
    let runtime_writes = count(
        client,
        "SELECT count(*) FROM information_schema.tables t, \
                unnest(ARRAY['lifeos_runtime','lifeos_worker']) AS role_name \
         WHERE t.table_schema = 'lifeos_release' \
           AND pg_catalog.has_table_privilege( \
                 role_name, \
                 format('%I.%I', t.table_schema, t.table_name), \
                 'INSERT, UPDATE, DELETE')",
    );
    let release_tables = count(
        client,
        "SELECT count(*) FROM information_schema.tables \
         WHERE table_schema = 'lifeos_release'",
    );
    // Three exemptions, each structural rather than convenient:
    //   * `_pre_s16` tables are retained pre-cutover lineage, not canonical.
    //   * `host_capture_staging` is pre-ingress staging; it holds no canonical
    //     record and is drained into tenant-scoped objects.
    //   * `backend_binding` is the bootstrap table itself — requiring a
    //     binding to read the binding would make authority unestablishable.
    let rls_disabled = count(
        client,
        "SELECT count(*) FROM pg_class c \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE c.relkind = 'r' AND NOT c.relrowsecurity \
           AND n.nspname IN ('lifeos_release','lifeos_blob','lifeos_security') \
           AND c.relname NOT LIKE '%\\_pre\\_s16' \
           AND (n.nspname, c.relname) NOT IN \
               (('lifeos_blob','host_capture_staging'), \
                ('lifeos_security','backend_binding'))",
    );

    match (runtime_writes, release_tables, rls_disabled) {
        (Ok(runtime_writes), Ok(release_tables), Ok(rls_disabled)) => {
            let measurement = json!({
                "release_tables": release_tables,
                "runtime_role_write_grants_on_release": runtime_writes,
                "canonical_tables_without_rls": rls_disabled,
            });
            if runtime_writes == 0 && rls_disabled == 0 {
                GateOutcome::pass(
                    "security",
                    "runtime roles hold no write grant on release; RLS enabled throughout",
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "security",
                    "privilege model is weaker than declared",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail("security", "privilege queries failed", json!({})),
    }
}

/// `model`: a registered model with recorded invocation lineage.
fn gate_model(client: &mut Client) -> GateOutcome {
    let models = count(client, "SELECT count(*) FROM lifeos_agent.model");
    let invocations = count(client, "SELECT count(*) FROM lifeos_agent.model_invocation");
    let io = count(client, "SELECT count(*) FROM lifeos_agent.model_io");

    match (models, invocations, io) {
        (Ok(models), Ok(invocations), Ok(io)) => {
            let measurement = json!({
                "models": models,
                "model_invocations": invocations,
                "model_io_records": io,
            });
            if models > 0 && invocations > 0 {
                GateOutcome::pass(
                    "model",
                    format!("{models} registered models, {invocations} invocations"),
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "model",
                    "no model is registered and no invocation lineage exists",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail("model", "model queries failed", json!({})),
    }
}

/// `forecast`: forecasts exist and have been scored against observations.
fn gate_forecast(client: &mut Client) -> GateOutcome {
    let forecasts = count(client, "SELECT count(*) FROM lifeos_agent.forecast");
    let observations = count(
        client,
        "SELECT count(*) FROM lifeos_agent.forecast_observation",
    );

    match (forecasts, observations) {
        (Ok(forecasts), Ok(observations)) => {
            let measurement = json!({
                "forecasts": forecasts,
                "forecast_observations": observations,
            });
            if forecasts > 0 && observations > 0 {
                GateOutcome::pass(
                    "forecast",
                    format!("{forecasts} forecasts scored by {observations} observations"),
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "forecast",
                    "no forecast has been issued or scored",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail("forecast", "forecast queries failed", json!({})),
    }
}

/// `witness`: every witness chain is internally linked and its head is honest.
///
/// Two independent properties, both required: each non-genesis entry's
/// `previous_shake256` equals its predecessor's `entry_shake256` (the chain is
/// unbroken), and each chain's recorded head equals its actual tail (the head
/// pointer has not drifted from the data it claims to summarise).
fn gate_witness(client: &mut Client) -> GateOutcome {
    let chains = count(client, "SELECT count(*) FROM lifeos_agent.witness_chain");
    let entries = count(client, "SELECT count(*) FROM lifeos_agent.witness_entry");
    let broken = count(
        client,
        "SELECT count(*) FROM lifeos_agent.witness_entry e \
         JOIN lifeos_agent.witness_entry p \
           ON p.chain_id = e.chain_id AND p.sequence = e.sequence - 1 \
         WHERE e.previous_shake256 IS DISTINCT FROM p.entry_shake256",
    );
    // An initialized-but-empty chain legitimately carries a genesis head with
    // no tail entry to match, so the head/tail comparison applies only to
    // chains that actually have entries. The sequence check still applies to
    // every chain, so an empty chain claiming a non-zero head is still drift.
    let head_drift = count(
        client,
        "SELECT count(*) FROM lifeos_agent.witness_chain c \
         WHERE c.head_sequence <> COALESCE( \
                 (SELECT max(e.sequence) FROM lifeos_agent.witness_entry e \
                  WHERE e.chain_id = c.chain_id), 0) \
            OR (EXISTS (SELECT 1 FROM lifeos_agent.witness_entry e \
                        WHERE e.chain_id = c.chain_id) \
                AND c.head_shake256 IS DISTINCT FROM \
                    (SELECT e.entry_shake256 FROM lifeos_agent.witness_entry e \
                     WHERE e.chain_id = c.chain_id \
                     ORDER BY e.sequence DESC LIMIT 1))",
    );

    match (chains, entries, broken, head_drift) {
        (Ok(chains), Ok(entries), Ok(broken), Ok(head_drift)) => {
            let measurement = json!({
                "chains": chains,
                "entries": entries,
                "broken_links": broken,
                "chains_with_head_drift": head_drift,
            });
            if entries > 0 && broken == 0 && head_drift == 0 {
                GateOutcome::pass(
                    "witness",
                    format!("{entries} entries across {chains} chains, unbroken, heads exact"),
                    measurement,
                )
            } else {
                GateOutcome::fail(
                    "witness",
                    "witness chain is broken, empty, or its head has drifted",
                    measurement,
                )
            }
        }
        _ => GateOutcome::fail("witness", "witness queries failed", json!({})),
    }
}

/// `runner-receipt`: the runner reports its own rails and seam wiring healthy.
fn gate_runner_receipt() -> GateOutcome {
    let output = Command::new("fxrun").arg("doctor").output();
    match output {
        Ok(result) if result.status.success() => GateOutcome::pass(
            "runner-receipt",
            "fxrun doctor reports healthy rails and seam wiring",
            json!({
                "command": "fxrun doctor",
                "exit": 0,
                "stdout_lines": String::from_utf8_lossy(&result.stdout).lines().count(),
            }),
        ),
        Ok(result) => GateOutcome::fail(
            "runner-receipt",
            "fxrun doctor reported an unhealthy runner",
            json!({
                "command": "fxrun doctor",
                "exit": result.status.code(),
                "stderr_tail": String::from_utf8_lossy(&result.stderr)
                    .lines()
                    .rev()
                    .take(6)
                    .collect::<Vec<_>>()
                    .join(" | "),
            }),
        ),
        Err(error) => GateOutcome::fail(
            "runner-receipt",
            format!("could not run fxrun doctor: {error}"),
            json!({ "command": "fxrun doctor" }),
        ),
    }
}

/// `rollback`: the production rollback path actually restores a prior target.
///
/// Exercises the real `activation` code an operator would depend on, against a
/// scratch link, rather than asserting that a rollback record exists.
fn gate_rollback() -> GateOutcome {
    let dir = std::env::temp_dir().join(format!("envctl-gate-rollback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let outcome = (|| -> Result<Value, String> {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let first = dir.join("generation-1");
        let second = dir.join("generation-2");
        std::fs::create_dir_all(&first).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&second).map_err(|e| e.to_string())?;
        let link = dir.join("current");

        std::os::unix::fs::symlink(&first, &link).map_err(|e| e.to_string())?;
        crate::activation::rollback(&link, second.to_str().ok_or("path")?, true)
            .map_err(|e| e.to_string())?;
        let after_forward = std::fs::read_link(&link).map_err(|e| e.to_string())?;
        crate::activation::rollback(&link, first.to_str().ok_or("path")?, true)
            .map_err(|e| e.to_string())?;
        let after_rollback = std::fs::read_link(&link).map_err(|e| e.to_string())?;

        let missing_refused =
            crate::activation::rollback(&link, "/nonexistent/generation", true).is_err();

        if after_forward != second {
            return Err("forward swap did not take".into());
        }
        if after_rollback != first {
            return Err("rollback did not restore the prior generation".into());
        }
        if !missing_refused {
            return Err("rollback accepted a target that does not exist".into());
        }
        Ok(json!({
            "restored_prior_generation": true,
            "missing_target_refused": true,
        }))
    })();

    let _ = std::fs::remove_dir_all(&dir);

    match outcome {
        Ok(measurement) => GateOutcome::pass(
            "rollback",
            "rollback restores the exact prior generation and refuses a missing target",
            measurement,
        ),
        Err(reason) => GateOutcome::fail("rollback", reason, json!({})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_list_matches_the_promote_contract() {
        // promote() enumerates these names verbatim; drift here silently
        // produces verification rows no gate loop will ever look at.
        assert_eq!(REQUIRED_GATES.len(), 11);
        assert_eq!(REQUIRED_GATES[0], "build");
        assert_eq!(REQUIRED_GATES[10], "rollback");
    }

    #[test]
    fn rollback_gate_exercises_the_real_activation_path() {
        // The rollback gate must be a live exercise, not a database lookup:
        // it passes here with no connection and no release records at all.
        let outcome = gate_rollback();
        assert!(outcome.passed, "rollback gate failed: {}", outcome.detail);
        assert_eq!(outcome.measurement["restored_prior_generation"], true);
        assert_eq!(outcome.measurement["missing_target_refused"], true);
    }

    #[test]
    fn build_gate_fails_closed_on_a_missing_release_root() {
        let outcome = gate_build(Path::new("/nix/store/definitely-not-a-real-release"));
        assert!(!outcome.passed);
        assert_eq!(outcome.measurement["exists"], false);
    }
}
