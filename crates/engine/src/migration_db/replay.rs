//! Replay verification (DATABASE_FEATURE_SPEC §Replay): recompute every recorded
//! hash from its recorded source and fail on any mismatch. `verify-only` is
//! implemented here; `dry-run-plan` renders the recipe as an operation plan;
//! `execute-again` is the pipeline's job (destructive replay requires approval,
//! so the engine refuses it without one).

use super::model::*;
use super::store;
use super::{canonical_json, sha256_hex, MigrationDb, MigrationDbError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayMode {
    VerifyOnly,
    DryRunPlan,
    ExecuteAgain,
}

impl ReplayMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "verify-only" => Ok(Self::VerifyOnly),
            "dry-run-plan" => Ok(Self::DryRunPlan),
            "execute-again" => Ok(Self::ExecuteAgain),
            other => Err(MigrationDbError::Validation(format!(
                "invalid replay mode: {other} (verify-only | dry-run-plan | execute-again)"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCheck {
    pub name: String,
    pub status: ValidationStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayReport {
    pub run_id: String,
    pub mode: ReplayMode,
    pub ok: bool,
    pub checks: Vec<ReplayCheck>,
    /// dry-run-plan: the operations that would run, in recipe order.
    pub plan: Option<Value>,
}

impl MigrationDb {
    pub fn replay(
        &self,
        run_id: &str,
        mode: ReplayMode,
        verify_files: bool,
    ) -> Result<ReplayReport> {
        match mode {
            ReplayMode::VerifyOnly => self.replay_verify(run_id, verify_files),
            ReplayMode::DryRunPlan => self.replay_dry_run_plan(run_id, verify_files),
            ReplayMode::ExecuteAgain => Err(MigrationDbError::ApprovalRequired(
                "execute-again is destructive replay: run it through the pipeline with an \
                 approved R3+ operation, not the engine verify surface"
                    .into(),
            )),
        }
    }

    fn replay_dry_run_plan(&self, run_id: &str, verify_files: bool) -> Result<ReplayReport> {
        let mut report = self.replay_verify(run_id, verify_files)?;
        let run = self.run(run_id)?;
        let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?;
        report.mode = ReplayMode::DryRunPlan;
        report.plan = Some(
            recipe
                .recipe_json
                .get("steps")
                .cloned()
                .unwrap_or(Value::Null),
        );
        Ok(report)
    }

    /// Recompute-and-compare every hash the spec lists: target descriptor, recipe,
    /// contract, package, command hashes, the full event chain, evidence hashes,
    /// artifact hashes, approval decisions, and the reproducibility hash.
    pub fn replay_verify(&self, run_id: &str, verify_files: bool) -> Result<ReplayReport> {
        let run = self.run(run_id)?;
        let mut checks: Vec<ReplayCheck> = Vec::new();
        let mut check = |name: &str, ok: bool, detail: String| {
            checks.push(ReplayCheck {
                name: name.to_string(),
                status: if ok {
                    ValidationStatus::Pass
                } else {
                    ValidationStatus::Fail
                },
                detail,
            });
        };

        // Identity hashes recomputed from their recorded sources.
        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let recomputed = sha256_hex(canonical_json(&target.descriptor_json).as_bytes());
        check(
            "target_descriptor_hash",
            recomputed == target.descriptor_hash,
            format!(
                "recorded {} recomputed {}",
                target.descriptor_hash, recomputed
            ),
        );
        let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?;
        let recomputed = sha256_hex(canonical_json(&recipe.recipe_json).as_bytes());
        check(
            "recipe_hash",
            recomputed == recipe.recipe_hash,
            format!("recorded {} recomputed {}", recipe.recipe_hash, recomputed),
        );
        let contract: ArtifactContract =
            self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?;
        let recomputed = sha256_hex(canonical_json(&contract.contract_json).as_bytes());
        check(
            "artifact_contract_hash",
            recomputed == contract.contract_hash,
            format!(
                "recorded {} recomputed {}",
                contract.contract_hash, recomputed
            ),
        );

        // Event chain: recompute every link.
        let events = self.events(run_id)?;
        let mut chain_ok = true;
        let mut chain_detail = format!("{} events", events.len());
        let mut previous: Option<String> = None;
        for ev in &events {
            if ev.previous_event_hash != previous {
                chain_ok = false;
                chain_detail = format!("event {} previous-hash link broken", ev.event_seq);
                break;
            }
            let mut clean = ev.clone();
            clean.event_hash = None;
            let body = serde_json::to_value(&clean)?;
            let material = format!(
                "{}\n{}",
                previous.clone().unwrap_or_default(),
                canonical_json(&body)
            );
            let recomputed = sha256_hex(material.as_bytes());
            if Some(&recomputed) != ev.event_hash.as_ref() {
                chain_ok = false;
                chain_detail = format!("event {} hash mismatch", ev.event_seq);
                break;
            }
            previous = ev.event_hash.clone();
        }
        check("event_chain", chain_ok, chain_detail);

        // Command hashes: recorded hash must match the recorded redacted command.
        let ops = self.operations(run_id)?;
        let bad_cmd = ops
            .iter()
            .filter(|o| match (&o.command_redacted, &o.command_hash) {
                (Some(cmd), Some(hash)) => &sha256_hex(cmd.as_bytes()) != hash,
                (None, Some(_)) => true,
                _ => false,
            })
            .count();
        check(
            "command_hashes",
            bad_cmd == 0,
            format!("{bad_cmd} mismatched of {} operations", ops.len()),
        );

        // Evidence + artifact hashes (recorded; optionally re-hashed from disk).
        let evidence = self.evidence(run_id)?;
        let missing = evidence.iter().filter(|e| e.sha256.is_none()).count();
        check(
            "evidence_hashes_recorded",
            missing == 0,
            format!(
                "{missing} of {} evidence rows missing sha256",
                evidence.len()
            ),
        );
        if verify_files {
            let mut bad = 0usize;
            let mut checked = 0usize;
            for ev in &evidence {
                if let (Some(recorded), true) =
                    (&ev.sha256, std::path::Path::new(&ev.uri).is_file())
                {
                    checked += 1;
                    let bytes = std::fs::read(&ev.uri)?;
                    if &sha256_hex(&bytes) != recorded {
                        bad += 1;
                    }
                }
            }
            check(
                "evidence_files_rehashed",
                bad == 0,
                format!("{bad} mismatched of {checked} on-disk evidence files"),
            );
        }
        let artifacts = self.artifacts(run_id)?;
        let missing = artifacts
            .iter()
            .filter(|a| a.content_hash.is_none())
            .count();
        check(
            "artifact_hashes_recorded",
            missing == 0,
            format!(
                "{missing} of {} artifacts missing content_hash",
                artifacts.len()
            ),
        );
        if verify_files {
            let mut bad = 0usize;
            let mut checked = 0usize;
            for a in &artifacts {
                if let (Some(recorded), Some(path)) = (&a.content_hash, &a.path) {
                    if std::path::Path::new(path).is_file() {
                        checked += 1;
                        let bytes = std::fs::read(path)?;
                        if &sha256_hex(&bytes) != recorded {
                            bad += 1;
                        }
                    }
                }
            }
            check(
                "artifact_files_rehashed",
                bad == 0,
                format!("{bad} mismatched of {checked} on-disk artifact files"),
            );
        }

        // Approvals: every approval decided (none open), decisions in the ledger.
        let approvals = self.approvals(run_id)?;
        let open = approvals
            .iter()
            .filter(|a| a.status == ApprovalStatus::Open)
            .count();
        check(
            "approval_decisions",
            open == 0,
            format!("{open} open of {} approvals", approvals.len()),
        );

        // Reproducibility hash (only recorded at completion).
        if let Some(recorded) = &run.reproducibility_hash {
            let last_hash = events
                .last()
                .and_then(|e| e.event_hash.clone())
                .unwrap_or_default();
            let material = format!(
                "{}\n{}\n{}\n{}\n{}",
                target.descriptor_hash,
                recipe.recipe_hash,
                contract.contract_hash,
                canonical_json(&run.tool_versions_json.clone().unwrap_or(Value::Null)),
                last_hash
            );
            let recomputed = sha256_hex(material.as_bytes());
            check(
                "reproducibility_hash",
                &recomputed == recorded,
                format!("recorded {recorded} recomputed {recomputed}"),
            );
        }

        let ok = checks.iter().all(|c| c.status == ValidationStatus::Pass);
        Ok(ReplayReport {
            run_id: run_id.to_string(),
            mode: ReplayMode::VerifyOnly,
            ok,
            checks,
            plan: None,
        })
    }
}
