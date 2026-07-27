//! Replay verification (DATABASE_FEATURE_SPEC §Replay): recompute every recorded
//! hash from its recorded source and fail on any mismatch. `verify-only` is
//! implemented here; `dry-run-plan` renders the recipe as an operation plan;
//! `execute-again` is the pipeline's job (destructive replay requires approval,
//! so the engine refuses it without one).

use super::model::*;
use super::store;
use super::{
    canonical_json, now_utc, sha256_hex, MigrationDb, MigrationDbError, Result,
};
use super::views::ReplayReadinessRow;
use serde_json::Map;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashSet, BTreeMap};
use std::path::{Path, PathBuf};

const REPLAY_BLOCKED_REF_PARTS: &[&str] = &[".env", "secrets", "private_keys"];
const REPLAY_BLOCKED_REF_SUFFIXES: &[&str] = &[".pem", ".key"];
const REPLAY_NON_DETERMINISTIC_OPERATION_TYPES: &[&str] = &[
    "external.apply",
    "manual_operator",
    "shell.exec",
    "target.mutate",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayMode {
    VerifyOnly,
    DryRunPlan,
    ExecuteAgain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayRequestMode {
    DryRun,
    Apply,
}

impl ReplayRequestMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "dry-run" | "dry_run" => Ok(Self::DryRun),
            "apply" => Ok(Self::Apply),
            other => Err(MigrationDbError::Validation(format!(
                "invalid replay mode: {other} (dry-run | apply)"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRequest {
    pub replay_id: String,
    pub run_id: String,
    pub mode: ReplayRequestMode,
    pub requested_by: String,
    #[serde(default)]
    pub operation_ids: Vec<String>,
    pub target_descriptor_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplayResultStatus {
    Pass,
    Fail,
    Blocked,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHashCheck {
    pub uri: Option<String>,
    pub expected_sha256: Option<String>,
    pub actual_sha256: Option<String>,
    pub status: String,
    pub blocked: bool,
    pub kind: Option<String>,
    pub operation_id: Option<String>,
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHashStatus {
    pub reproducibility_hash_matches: bool,
    pub stored_reproducibility_hash: Option<String>,
    pub recomputed_reproducibility_hash: String,
    pub evidence_matches: usize,
    pub evidence_mismatches: Vec<ReplayHashCheck>,
    pub evidence_missing: Vec<ReplayHashCheck>,
    pub artifact_matches: usize,
    pub artifact_mismatches: Vec<ReplayHashCheck>,
    pub artifact_missing: Vec<ReplayHashCheck>,
    pub blocked_refs: Vec<ReplayHashCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEventChain {
    pub run_id: String,
    pub event_count: usize,
    pub chain_valid: bool,
    pub errors: Vec<String>,
    pub head_event_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayNonDeterministicOperation {
    pub operation_id: String,
    pub operation_type: String,
    pub risk: Risk,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayPlanRow {
    pub operation_id: String,
    pub operation_type: String,
    pub phase: Option<String>,
    pub status: OpStatus,
    pub risk: Risk,
    pub idempotency_key: String,
    pub command_hash: Option<String>,
    pub output_ref: Option<String>,
    pub recipe_phase: Option<String>,
    pub approval_gate: bool,
    pub checkpoint_refs: Vec<Checkpoint>,
    pub replay_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub schema_version: String,
    pub replay_id: String,
    pub run_id: String,
    pub mode: ReplayRequestMode,
    pub requested_by: String,
    pub completed_at_utc: String,
    pub status: ReplayResultStatus,
    pub replay_input_hash: String,
    pub replay_input: Value,
    pub readiness: ReplayReadinessRow,
    pub event_chain: ReplayEventChain,
    pub recipe_operations: Vec<Value>,
    pub operation_replay_plan: Vec<ReplayPlanRow>,
    pub hash_status: ReplayHashStatus,
    pub missing_evidence: Vec<ReplayHashCheck>,
    pub non_deterministic_operations: Vec<ReplayNonDeterministicOperation>,
    pub required_approvals: Vec<Value>,
    pub safe_next_action: String,
    pub errors: Vec<String>,
    pub event_refs: Vec<String>,
    pub artifact_refs: Vec<String>,
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

    pub fn replay_request(
        &self,
        request: ReplayRequest,
        verify_files: bool,
    ) -> Result<ReplayResult> {
        let run = self.run(&request.run_id)?;
        if let Some(target_descriptor_id) = &request.target_descriptor_id {
            if target_descriptor_id != &run.target_id {
                return Err(MigrationDbError::Validation(format!(
                    "target_descriptor_id mismatch: requested {target_descriptor_id}, run has {}",
                    run.target_id
                )));
            }
        }

        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?;
        let contract: ArtifactContract =
            self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?;
        let all_ops = self.operations(&request.run_id)?;
        let operations = select_operations(&all_ops, &request.operation_ids)?;
        let operation_ids: Vec<String> = operations.iter().map(|op| op.id.clone()).collect();
        let operation_ids_set: HashSet<String> = operation_ids.iter().cloned().collect();
        let evidence = {
            let all = self.evidence(&request.run_id)?;
            if request.operation_ids.is_empty() {
                all
            } else {
                all.into_iter()
                    .filter(|row| {
                        row.operation_id
                            .as_ref()
                            .is_some_and(|id| operation_ids_set.contains(id))
                    })
                    .collect()
            }
        };
        let artifacts = {
            let all = self.artifacts(&request.run_id)?;
            if request.operation_ids.is_empty() {
                all
            } else {
                all.into_iter()
                    .filter(|row| {
                        row.generated_by_operation_id
                            .as_ref()
                            .is_some_and(|id| operation_ids_set.contains(id))
                    })
                    .collect()
            }
        };
        let approvals = {
            let all = self.approvals(&request.run_id)?;
            if request.operation_ids.is_empty() {
                all
            } else {
                all.into_iter()
                    .filter(|row| operation_ids_set.contains(&row.operation_id))
                    .collect()
            }
        };
        let events = self.events(&request.run_id)?;
        let checkpoints = self.checkpoints(&request.run_id)?;
        let readiness = self.view_replay_readiness(&request.run_id)?;

        let mut event_chain_errors = Vec::new();
        let mut previous = None;
        for ev in &events {
            if ev.previous_event_hash != previous {
                event_chain_errors.push(format!(
                    "event {} previous-hash link broken",
                    ev.event_seq
                ));
            }
            let mut clean = ev.clone();
            clean.event_hash = None;
            let body = serde_json::to_value(&clean)?;
            let material = format!("{}\n{}", previous.unwrap_or_default(), canonical_json(&body));
            let recomputed = sha256_hex(material.as_bytes());
            if Some(&recomputed) != ev.event_hash.as_ref() {
                event_chain_errors.push(format!("event {} hash mismatch", ev.event_seq));
            }
            previous = ev.event_hash.clone();
        }
        let chain_valid = event_chain_errors.is_empty();

        let command_mismatches = operations
            .iter()
            .filter(|o| match (&o.command_redacted, &o.command_hash) {
                (Some(cmd), Some(hash)) => &sha256_hex(cmd.as_bytes()) != hash,
                (None, Some(_)) => true,
                _ => false,
            })
            .count();
        if command_mismatches != 0 {
            event_chain_errors.push(format!(
                "{command_mismatches} command hash mismatch(es)",
            ));
        }

        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let evidence_checks = evidence
            .iter()
            .map(|row| {
                let mut check = file_hash_check(
                    &row.uri,
                    row.sha256.as_deref(),
                    &base,
                    verify_files,
                );
                check.kind = Some(row.evidence_kind.clone());
                check.operation_id = row.operation_id.clone();
                check
            })
            .collect::<Vec<_>>();
        let artifact_checks = artifacts
            .iter()
            .map(|row| {
                let mut check = file_hash_check(
                    row.path.as_deref().unwrap_or_default(),
                    row.content_hash.as_deref(),
                    &base,
                    verify_files,
                );
                check.artifact_id = Some(row.artifact_id.clone());
                check
            })
            .collect::<Vec<_>>();
        let mut missing_evidence = Vec::new();
        let mut evidence_mismatches = Vec::new();
        let mut evidence_matches = 0usize;
        let mut evidence_missing = Vec::new();
        for item in &evidence_checks {
            if item.status == "match" {
                evidence_matches += 1;
            } else if item.status == "missing" || item.status == "missing_hash" {
                evidence_missing.push(item.clone());
            } else if item.status == "mismatch" {
                evidence_mismatches.push(item.clone());
            }
            if item.status == "missing" || item.status == "missing_hash" {
                missing_evidence.push(item.clone());
            }
        }

        let mut artifact_mismatches = Vec::new();
        let mut artifact_missing = Vec::new();
        let mut artifact_matches = 0usize;
        for item in &artifact_checks {
            if item.status == "match" {
                artifact_matches += 1;
            } else if item.status == "missing" || item.status == "missing_hash" {
                artifact_missing.push(item.clone());
            } else if item.status == "mismatch" {
                artifact_mismatches.push(item.clone());
            }
        }

        let blocked_refs = evidence_checks
            .iter()
            .chain(artifact_checks.iter())
            .filter(|c| c.status == "blocked_ref" || c.status == "out_of_scope")
            .cloned()
            .collect::<Vec<_>>();
        let recipe_operations = extract_recipe_operations(&recipe.recipe_json);
        let non_deterministic = operations
            .iter()
            .filter_map(non_deterministic_operation)
            .collect::<Vec<_>>();
        let required_approvals = approvals
            .iter()
            .filter(|a| a.status == ApprovalStatus::Open)
            .map(|approval| {
                let op = all_ops
                    .iter()
                    .find(|op| op.id == approval.operation_id)
                    .map_or("unknown", |op| op.operation_type.as_str());
                serde_json::json!({
                    "approval_id": approval.id,
                    "operation_id": approval.operation_id,
                    "operation_type": op,
                    "risk": approval.risk.as_str(),
                    "reason": approval.reason,
                    "requested_by": approval.requested_by,
                })
            })
            .collect::<Vec<_>>();

        let artifact_refs = artifacts
            .iter()
            .map(|item| item.artifact_id.clone())
            .collect();
        let event_refs = events.iter().map(|item| item.id.clone()).collect();

        let mut recipe_map: BTreeMap<String, Value> = BTreeMap::new();
        for item in &recipe_operations {
            if let Some(operation_type) = item.get("operation_type").and_then(Value::as_str) {
                recipe_map.insert(operation_type.to_string(), item.clone());
            }
        }
        let mut checkpoints_by_op = BTreeMap::new();
        for checkpoint in checkpoints {
            if let Some(op_id) = checkpoint.operation_id.clone() {
                checkpoints_by_op
                    .entry(op_id)
                    .or_insert_with(Vec::new)
                    .push(checkpoint);
            }
        }
        let operation_replay_plan = operations
            .iter()
            .map(|op| {
                let recipe_op = recipe_map.get(&op.operation_type);
                let approval_gate = recipe_op
                    .and_then(|item| item.get("approval_gate"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let recipe_phase = recipe_op
                    .and_then(|item| item.get("phase_id"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let checkpoint_refs = checkpoints_by_op
                    .get(&op.id)
                    .cloned()
                    .unwrap_or_default();
                let replay_action = if op.status == OpStatus::Succeeded {
                    "verify_only"
                } else {
                    "resume_from_checkpoint"
                };
                ReplayPlanRow {
                    operation_id: op.id.clone(),
                    operation_type: op.operation_type.clone(),
                    phase: op.phase.clone(),
                    status: op.status,
                    risk: op.risk,
                    idempotency_key: op.idempotency_key.clone(),
                    command_hash: op.command_hash.clone(),
                    output_ref: op.output_ref.clone(),
                    recipe_phase,
                    approval_gate,
                    checkpoint_refs,
                    replay_action: replay_action.to_string(),
                }
            })
            .collect::<Vec<_>>();

        let recomputed_reproducibility_hash = {
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
            sha256_hex(material.as_bytes())
        };
        let stored_reproducibility_hash = run.reproducibility_hash.clone();
        let reproducibility_hash_matches =
            matches!((&stored_reproducibility_hash, recomputed_reproducibility_hash.as_str()),
                (Some(stored), recomputed) if stored == recomputed);

        let has_blocking_refs = evidence_checks
            .iter()
            .chain(&artifact_checks)
            .any(|check| check.status == "blocked_ref" || check.status == "out_of_scope");

        let mut errors = Vec::new();
        if !chain_valid {
            errors.extend(event_chain_errors.iter().cloned());
        }
        if command_mismatches != 0 {
            errors.push(format!(
                "{command_mismatches} command hash mismatches"
            ));
        }
        if !evidence_mismatches.is_empty() {
            errors.push(format!(
                "{} evidence hash mismatches",
                evidence_mismatches.len()
            ));
        }
        if !artifact_mismatches.is_empty() {
            errors.push(format!(
                "{} artifact hash mismatches",
                artifact_mismatches.len()
            ));
        }
        if has_blocking_refs {
            errors.push("blocked or out-of-scope replay reference".to_string());
        }
        if !reproducibility_hash_matches {
            errors.push("run reproducibility hash mismatch".to_string());
        }
        if !required_approvals.is_empty() && request.mode == ReplayRequestMode::Apply {
            errors.push("apply replay requires closed approvals".to_string());
        }
        if !non_deterministic.is_empty() && request.mode == ReplayRequestMode::Apply {
            errors.push("apply replay requires manual handling for non-deterministic operations".to_string());
        }

        let status = if matches!(request.mode, ReplayRequestMode::Apply) && !errors.is_empty() {
            ReplayResultStatus::Blocked
        } else if (!evidence_missing.is_empty() || !artifact_missing.is_empty())
            || (!required_approvals.is_empty())
            || !non_deterministic.is_empty()
        {
            if chain_valid && errors.is_empty() {
                ReplayResultStatus::Partial
            } else {
                ReplayResultStatus::Blocked
            }
        } else if errors.is_empty() {
            ReplayResultStatus::Pass
        } else {
            ReplayResultStatus::Fail
        };

        let replay_input = serde_json::json!({
            "target_descriptor": {
                "id": target.id,
                "target_id": target.target_id,
                "descriptor_hash": target.descriptor_hash,
                "safety_mode": target.safety_mode,
                "max_auto_risk": target.max_auto_risk.as_str(),
            },
            "artifact_contract": {
                "id": contract.id,
                "contract_hash": contract.contract_hash,
            },
            "recipe": {
                "id": recipe.id,
                "recipe_hash": recipe.recipe_hash,
                "operation_count": recipe_operations.len(),
            },
            "run": {
                "id": run.id,
                "status": run.status.as_str(),
                "human_mode": run.human_mode.as_str(),
                "sandbox_policy": run.sandbox_policy,
                "approval_policy": run.approval_policy,
                "tool_versions": run.tool_versions_json.clone().unwrap_or(Value::Null),
                "reproducibility_hash": run.reproducibility_hash,
            },
            "operation_ids": operation_ids,
            "event_chain_head": events.last().and_then(|ev| ev.event_hash.clone()),
        });
        let replay_input_hash = sha256_hex(canonical_json(&replay_input).as_bytes());
        let safe_next_action = safe_next_action(
            &request,
            status,
            &required_approvals,
            &non_deterministic,
            has_blocking_refs,
            &evidence_mismatches,
            &artifact_mismatches,
        );

        let hash_status = ReplayHashStatus {
            reproducibility_hash_matches,
            stored_reproducibility_hash,
            recomputed_reproducibility_hash,
            evidence_matches,
            evidence_mismatches,
            evidence_missing,
            artifact_matches,
            artifact_mismatches,
            artifact_missing,
            blocked_refs,
        };
        let event_chain = ReplayEventChain {
            run_id: run.id.clone(),
            event_count: events.len(),
            chain_valid,
            errors: event_chain_errors,
            head_event_hash: events.last().and_then(|ev| ev.event_hash.clone()),
        };

        Ok(ReplayResult {
            schema_version: "1.0".to_string(),
            replay_id: request.replay_id,
            run_id: run.id,
            mode: request.mode,
            requested_by: request.requested_by,
            completed_at_utc: now_utc(),
            status,
            replay_input_hash,
            replay_input,
            readiness,
            event_chain,
            recipe_operations,
            operation_replay_plan,
            hash_status,
            missing_evidence,
            non_deterministic_operations: non_deterministic,
            required_approvals,
            safe_next_action,
            errors,
            event_refs,
            artifact_refs,
        })
    }
}

fn select_operations(
    operations: &[Operation],
    operation_ids: &[String],
) -> Result<Vec<Operation>> {
    if operation_ids.is_empty() {
        return Ok(operations.to_vec());
    }

    let requested: HashSet<&str> = operation_ids.iter().map(String::as_str).collect();
    let selected = operations
        .iter()
        .filter(|op| requested.contains(op.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let found: HashSet<&str> = selected.iter().map(|op| op.id.as_str()).collect();
    let missing: Vec<_> = operation_ids
        .iter()
        .filter(|id| !found.contains(id.as_str()))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        return Err(MigrationDbError::Validation(format!(
            "unknown operation ids for replay: {}",
            missing.join(", ")
        )));
    }
    Ok(selected)
}

fn extract_recipe_operations(recipe_json: &Value) -> Vec<Value> {
    if let Some(steps) = recipe_json.get("steps").and_then(Value::as_array) {
        return steps.clone();
    }

    let mut out = Vec::new();
    if let Some(phases) = recipe_json.get("phases").and_then(Value::as_array) {
        for phase in phases {
            let phase_id = phase.get("phase_id").cloned();
            let approval_gate = phase.get("approval_gate").cloned();
            if let Some(ops) = phase.get("operations").and_then(Value::as_array) {
                for op in ops {
                    if let Some(item) = op.as_object() {
                        let mut row: Map<String, Value> = item.clone();
                        if let Some(id) = &phase_id {
                            row.insert("phase_id".to_string(), id.clone());
                        }
                        if let Some(flag) = &approval_gate {
                            row.insert("approval_gate".to_string(), flag.clone());
                        }
                        out.push(Value::Object(row));
                    }
                }
            }
        }
    }
    out
}

fn is_blocked_reference(raw_uri: &str) -> bool {
    let normalized = raw_uri.replace('\\', "/").to_lowercase();
    let part_hit = normalized
        .split('/')
        .filter(|part| !part.is_empty())
        .any(|part| REPLAY_BLOCKED_REF_PARTS.iter().any(|blocked| part == *blocked));
    let suffix_hit = REPLAY_BLOCKED_REF_SUFFIXES
        .iter()
        .any(|suffix| normalized.ends_with(suffix));
    part_hit || suffix_hit
}

fn file_hash_check(uri: &str, expected: Option<&str>, base: &Path, verify_files: bool) -> ReplayHashCheck {
    if uri.is_empty() {
        return ReplayHashCheck {
            uri: None,
            expected_sha256: expected.map(ToString::to_string),
            actual_sha256: None,
            status: "missing_path".to_string(),
            blocked: false,
            kind: None,
            operation_id: None,
            artifact_id: None,
        };
    }
    if is_blocked_reference(uri) {
        return ReplayHashCheck {
            uri: Some(uri.to_string()),
            expected_sha256: expected.map(ToString::to_string),
            actual_sha256: None,
            status: "blocked_ref".to_string(),
            blocked: true,
            kind: None,
            operation_id: None,
            artifact_id: None,
        };
    }

    let base = base
        .canonicalize()
        .unwrap_or_else(|_| base.to_path_buf());
    let candidate = {
        let raw = PathBuf::from(uri.replace('\\', "/"));
        if raw.is_absolute() {
            raw
        } else {
            base.join(raw)
        }
    };
    if !candidate.starts_with(&base) {
        return ReplayHashCheck {
            uri: Some(uri.to_string()),
            expected_sha256: expected.map(ToString::to_string),
            actual_sha256: None,
            status: "out_of_scope".to_string(),
            blocked: true,
            kind: None,
            operation_id: None,
            artifact_id: None,
        };
    }

    let Some(expected_hash) = expected else {
        return ReplayHashCheck {
            uri: Some(uri.to_string()),
            expected_sha256: None,
            actual_sha256: None,
            status: "missing_hash".to_string(),
            blocked: false,
            kind: None,
            operation_id: None,
            artifact_id: None,
        };
    };

    if !candidate.is_file() {
        return ReplayHashCheck {
            uri: Some(uri.to_string()),
            expected_sha256: Some(expected_hash.to_string()),
            actual_sha256: None,
            status: "missing".to_string(),
            blocked: false,
            kind: None,
            operation_id: None,
            artifact_id: None,
        };
    }
    let actual = if verify_files {
        std::fs::read(&candidate).ok().map(|bytes| sha256_hex(&bytes))
    } else {
        None
    };
    let status = match actual.as_deref() {
        Some(actual_hash) => {
            if actual_hash == expected_hash {
                "match"
            } else {
                "mismatch"
            }
        }
        None => "missing",
    };
    ReplayHashCheck {
        uri: Some(uri.to_string()),
        expected_sha256: Some(expected_hash.to_string()),
        actual_sha256: actual,
        status: status.to_string(),
        blocked: false,
        kind: None,
        operation_id: None,
        artifact_id: None,
    }
}

fn non_deterministic_operation(op: &Operation) -> Option<ReplayNonDeterministicOperation> {
    if REPLAY_NON_DETERMINISTIC_OPERATION_TYPES
        .iter()
        .any(|name| op.operation_type == *name)
        || op
            .input_json
            .as_ref()
            .and_then(|json| json.get("non_deterministic"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || matches!(op.risk, Risk::R3 | Risk::R4 | Risk::R5)
    {
        Some(ReplayNonDeterministicOperation {
            operation_id: op.id.clone(),
            operation_type: op.operation_type.clone(),
            risk: op.risk,
            reason: op
                .input_json
                .as_ref()
                .and_then(|json| json.get("replay_note"))
                .and_then(Value::as_str)
                .unwrap_or("operation requires external or manual state")
                .to_string(),
        })
    } else {
        None
    }
}

fn safe_next_action(
    request: &ReplayRequest,
    status: ReplayResultStatus,
    required_approvals: &[Value],
    non_deterministic: &[ReplayNonDeterministicOperation],
    has_blocking_refs: bool,
    evidence_mismatches: &[ReplayHashCheck],
    artifact_mismatches: &[ReplayHashCheck],
) -> String {
    if !evidence_mismatches.is_empty() || !artifact_mismatches.is_empty() {
        return "stop: inspect hash mismatches before replay".to_string();
    }
    if has_blocking_refs {
        return "stop: remove blocked or out-of-scope replay references".to_string();
    }
    if !required_approvals.is_empty() {
        return "request human approval before apply replay".to_string();
    }
    if !non_deterministic.is_empty() {
        return "dry-run only: manual/non-deterministic operations require operator handling".to_string();
    }
    if request.mode == ReplayRequestMode::DryRun && matches!(status, ReplayResultStatus::Pass | ReplayResultStatus::Partial)
    {
        return "safe to produce replay command plan; apply remains approval-gated".to_string();
    }
    if request.mode == ReplayRequestMode::Apply && status == ReplayResultStatus::Pass {
        return "safe to re-run deterministic operations from recorded descriptors and hashes".to_string();
    }
    "stop: replay result is not clean".to_string()
}
