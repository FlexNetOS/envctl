//! Fail-closed migration replay and reproducibility verification.
//!
//! Replay reconstructs a plan from the immutable catalog and run ledger. It
//! never executes `command_redacted`: that field is evidence, not trusted code.
//! Apply mode means the reconstructed plan is eligible for the normal operation
//! pipeline; hashes, filesystem scope, approvals, and determinism must all pass.

use super::model::*;
use super::store;
use super::{canonical_json, now_utc, sha256_hex, MigrationDb, MigrationDbError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

const NON_DETERMINISTIC_OPERATION_TYPES: &[&str] = &[
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
    Apply,
    /// Retained for compatibility; callers must use the guarded apply surface.
    ExecuteAgain,
}

impl ReplayMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "verify-only" => Ok(Self::VerifyOnly),
            "dry-run" | "dry_run" | "dry-run-plan" => Ok(Self::DryRunPlan),
            "apply" => Ok(Self::Apply),
            "execute-again" => Ok(Self::ExecuteAgain),
            other => Err(MigrationDbError::Validation(format!(
                "invalid replay mode: {other} (verify-only | dry-run | dry-run-plan | apply)"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRequest {
    pub replay_id: String,
    pub run_id: String,
    pub mode: ReplayMode,
    pub requested_by: String,
    #[serde(default)]
    pub operation_ids: Vec<String>,
    pub target_descriptor_id: Option<String>,
    pub reason: Option<String>,
    /// Filesystem root within which evidence and artifacts may be re-hashed.
    pub replay_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCheck {
    pub name: String,
    pub status: ValidationStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashCheck {
    pub uri: String,
    pub expected_sha256: Option<String>,
    pub actual_sha256: Option<String>,
    pub status: String,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOperationPlan {
    pub operation_id: String,
    pub operation_type: String,
    pub phase: Option<String>,
    pub status: OpStatus,
    pub risk: Risk,
    pub idempotency_key: String,
    pub command_hash: Option<String>,
    pub output_ref: Option<String>,
    pub checkpoint_refs: Vec<Checkpoint>,
    pub replay_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredApproval {
    pub approval_id: String,
    pub operation_id: String,
    pub risk: Risk,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonDeterministicOperation {
    pub operation_id: String,
    pub operation_type: String,
    pub risk: Risk,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayReport {
    pub schema_version: String,
    pub replay_id: String,
    pub run_id: String,
    pub mode: ReplayMode,
    pub requested_by: String,
    pub completed_at_utc: String,
    pub status: String,
    pub ok: bool,
    pub replay_input_hash: String,
    pub replay_input: Value,
    pub checks: Vec<ReplayCheck>,
    pub hash_checks: Vec<FileHashCheck>,
    pub missing_evidence: Vec<FileHashCheck>,
    pub blocked_refs: Vec<FileHashCheck>,
    pub non_deterministic_operations: Vec<NonDeterministicOperation>,
    pub required_approvals: Vec<RequiredApproval>,
    pub operation_replay_plan: Vec<ReplayOperationPlan>,
    pub safe_next_action: String,
    pub errors: Vec<String>,
    /// Compatibility field for `migration run replay --mode dry-run-plan`.
    pub plan: Option<Value>,
}

impl MigrationDb {
    /// Compatibility replay surface used by `envctl migration run replay`.
    pub fn replay(
        &self,
        run_id: &str,
        mode: ReplayMode,
        verify_files: bool,
    ) -> Result<ReplayReport> {
        if matches!(mode, ReplayMode::ExecuteAgain) {
            return Err(MigrationDbError::ApprovalRequired(
                "execute-again is retired: use the guarded replay apply surface".into(),
            ));
        }
        let request = ReplayRequest {
            replay_id: format!("replay-{run_id}"),
            run_id: run_id.to_string(),
            mode,
            requested_by: "envctl".into(),
            operation_ids: Vec::new(),
            target_descriptor_id: None,
            reason: None,
            replay_root: verify_files.then(|| PathBuf::from(".")),
        };
        self.reproduce(request)
    }

    /// Reconstruct and verify a replay request. No stored command is executed.
    pub fn reproduce(&self, request: ReplayRequest) -> Result<ReplayReport> {
        validate_request(&request)?;
        let run = self.run(&request.run_id)?;
        if let Some(expected) = request.target_descriptor_id.as_deref() {
            if expected != run.target_id {
                return Err(MigrationDbError::Validation(format!(
                    "target descriptor {expected} does not match run target {}",
                    run.target_id
                )));
            }
        }

        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?;
        let contract: ArtifactContract =
            self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?;
        let package = contract
            .source_package_id
            .as_deref()
            .map(|id| self.must_get::<Package>(store::PACKAGES, id, "package"))
            .transpose()?;
        let operations = select_operations(self.operations(&run.id)?, &request.operation_ids)?;
        let selected: BTreeSet<&str> = operations.iter().map(|op| op.id.as_str()).collect();
        let events = self.events(&run.id)?;
        let evidence: Vec<_> = self
            .evidence(&run.id)?
            .into_iter()
            .filter(|row| {
                request.operation_ids.is_empty()
                    || row
                        .operation_id
                        .as_deref()
                        .is_some_and(|id| selected.contains(id))
            })
            .collect();
        let artifacts: Vec<_> = self
            .artifacts(&run.id)?
            .into_iter()
            .filter(|row| {
                request.operation_ids.is_empty()
                    || row
                        .generated_by_operation_id
                        .as_deref()
                        .is_some_and(|id| selected.contains(id))
            })
            .collect();
        let approvals = self.approvals(&run.id)?;
        let checkpoints = self.checkpoints(&run.id)?;

        let mut checks = Vec::new();
        catalog_check(
            &mut checks,
            "target_descriptor_hash",
            &target.descriptor_hash,
            &sha256_hex(canonical_json(&target.descriptor_json).as_bytes()),
        );
        catalog_check(
            &mut checks,
            "recipe_hash",
            &recipe.recipe_hash,
            &sha256_hex(canonical_json(&recipe.recipe_json).as_bytes()),
        );
        catalog_check(
            &mut checks,
            "artifact_contract_hash",
            &contract.contract_hash,
            &sha256_hex(canonical_json(&contract.contract_json).as_bytes()),
        );
        if let Some(package) = &package {
            push_check(
                &mut checks,
                "package_manifest_hash",
                !package.package_hash.is_empty() && package.manifest_json.is_object(),
                format!(
                    "recorded {} for package manifest {}",
                    package.package_hash, package.package_path
                ),
            );
        } else {
            checks.push(ReplayCheck {
                name: "package_manifest_hash".into(),
                status: ValidationStatus::Warn,
                detail: "artifact contract has no source package".into(),
            });
        }

        let (chain_ok, chain_detail, chain_head) = verify_event_chain(&events)?;
        push_check(&mut checks, "event_chain", chain_ok, chain_detail);
        let bad_commands = operations
            .iter()
            .filter(|op| match (&op.command_redacted, &op.command_hash) {
                (Some(command), Some(hash)) => !hash_equal(hash, &sha256_hex(command.as_bytes())),
                (None, Some(_)) => true,
                _ => false,
            })
            .count();
        push_check(
            &mut checks,
            "command_hashes",
            bad_commands == 0,
            format!(
                "{bad_commands} mismatched of {} operations",
                operations.len()
            ),
        );

        let mut hash_checks = Vec::new();
        for row in &evidence {
            hash_checks.push(check_file(
                &row.uri,
                row.sha256.as_deref(),
                request.replay_root.as_deref(),
            )?);
        }
        for row in &artifacts {
            match row.path.as_deref() {
                Some(path) => hash_checks.push(check_file(
                    path,
                    row.content_hash.as_deref(),
                    request.replay_root.as_deref(),
                )?),
                None => hash_checks.push(FileHashCheck {
                    uri: format!("artifact:{}", row.artifact_id),
                    expected_sha256: row.content_hash.clone(),
                    actual_sha256: None,
                    status: "missing_path".into(),
                    blocked: false,
                }),
            }
        }
        let missing_evidence: Vec<_> = hash_checks
            .iter()
            .filter(|check| {
                matches!(
                    check.status.as_str(),
                    "missing" | "missing-hash" | "missing-path"
                )
            })
            .cloned()
            .collect();
        let blocked_refs: Vec<_> = hash_checks
            .iter()
            .filter(|check| matches!(check.status.as_str(), "blocked-ref" | "out-of-scope"))
            .cloned()
            .collect();
        let mismatches = hash_checks
            .iter()
            .filter(|check| check.status == "mismatch")
            .count();
        push_check(
            &mut checks,
            "evidence_artifact_hashes",
            missing_evidence.is_empty() && blocked_refs.is_empty() && mismatches == 0,
            format!(
                "{mismatches} mismatches, {} missing, {} blocked/out-of-scope",
                missing_evidence.len(),
                blocked_refs.len()
            ),
        );

        let required_approvals: Vec<_> = approvals
            .iter()
            .filter(|approval| {
                approval.status == ApprovalStatus::Open
                    && selected.contains(approval.operation_id.as_str())
            })
            .map(|approval| RequiredApproval {
                approval_id: approval.id.clone(),
                operation_id: approval.operation_id.clone(),
                risk: approval.risk,
                reason: approval.reason.clone(),
            })
            .collect();
        let non_deterministic_operations = non_deterministic(&operations);
        let operation_replay_plan =
            operation_plan(&operations, &checkpoints, &request.operation_ids);

        let replay_input = json!({
            "target_descriptor": {
                "id": target.id,
                "target_id": target.target_id,
                "descriptor_hash": target.descriptor_hash,
                "safety_mode": target.safety_mode,
                "max_auto_risk": target.max_auto_risk,
            },
            "artifact_contract": {
                "id": contract.id,
                "contract_hash": contract.contract_hash,
            },
            "package_manifest": package.as_ref().map(|p| json!({
                "id": p.id,
                "package_path": p.package_path,
                "package_hash": p.package_hash,
            })),
            "recipe": {
                "id": recipe.id,
                "recipe_hash": recipe.recipe_hash,
            },
            "run": {
                "id": run.id,
                "status": run.status,
                "human_mode": run.human_mode,
                "sandbox_policy": run.sandbox_policy,
                "approval_policy": run.approval_policy,
                "tool_versions": run.tool_versions_json,
                "reproducibility_hash": run.reproducibility_hash,
            },
            "operation_ids": operations.iter().map(|op| &op.id).collect::<Vec<_>>(),
            "event_chain_head": chain_head,
        });
        let replay_input_hash = sha256_hex(canonical_json(&replay_input).as_bytes());

        if let Some(recorded) = &run.reproducibility_hash {
            let material = format!(
                "{}\n{}\n{}\n{}\n{}",
                target.descriptor_hash,
                recipe.recipe_hash,
                contract.contract_hash,
                canonical_json(&run.tool_versions_json.clone().unwrap_or(Value::Null)),
                chain_head.clone().unwrap_or_default()
            );
            catalog_check(
                &mut checks,
                "reproducibility_hash",
                recorded,
                &sha256_hex(material.as_bytes()),
            );
        } else {
            checks.push(ReplayCheck {
                name: "reproducibility_hash".into(),
                status: ValidationStatus::Fail,
                detail: "run has no reproducibility hash".into(),
            });
        }

        let mut errors: Vec<String> = checks
            .iter()
            .filter(|check| check.status == ValidationStatus::Fail)
            .map(|check| format!("{}: {}", check.name, check.detail))
            .collect();
        if matches!(request.mode, ReplayMode::Apply) {
            if !required_approvals.is_empty() {
                errors.push("apply replay requires closed approvals".into());
            }
            if !non_deterministic_operations.is_empty() {
                errors.push(
                    "apply replay requires manual handling for non-deterministic operations".into(),
                );
            }
        }
        let apply_blocked = matches!(request.mode, ReplayMode::Apply) && !errors.is_empty();
        let status = if apply_blocked {
            "blocked"
        } else if !errors.is_empty() {
            "fail"
        } else if !required_approvals.is_empty()
            || !non_deterministic_operations.is_empty()
            || !missing_evidence.is_empty()
        {
            "partial"
        } else {
            "pass"
        }
        .to_string();
        let safe_next_action = safe_next_action(
            &request,
            &status,
            mismatches,
            &blocked_refs,
            &required_approvals,
            &non_deterministic_operations,
        );
        let plan = matches!(request.mode, ReplayMode::DryRunPlan | ReplayMode::Apply)
            .then(|| serde_json::to_value(&operation_replay_plan).unwrap_or(Value::Null));

        Ok(ReplayReport {
            schema_version: "1.0".into(),
            replay_id: request.replay_id,
            run_id: request.run_id,
            mode: request.mode,
            requested_by: request.requested_by,
            completed_at_utc: now_utc(),
            status: status.clone(),
            ok: status == "pass" || (status == "partial" && request.mode != ReplayMode::Apply),
            replay_input_hash,
            replay_input,
            checks,
            hash_checks,
            missing_evidence,
            blocked_refs,
            non_deterministic_operations,
            required_approvals,
            operation_replay_plan,
            safe_next_action,
            errors,
            plan,
        })
    }
}

fn validate_request(request: &ReplayRequest) -> Result<()> {
    for (name, value) in [
        ("replay_id", request.replay_id.as_str()),
        ("run_id", request.run_id.as_str()),
        ("requested_by", request.requested_by.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(MigrationDbError::Validation(format!(
                "replay request missing required field: {name}"
            )));
        }
    }
    Ok(())
}

fn select_operations(all: Vec<Operation>, ids: &[String]) -> Result<Vec<Operation>> {
    if ids.is_empty() {
        return Ok(all);
    }
    let requested: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    let found: BTreeSet<&str> = all
        .iter()
        .filter(|op| requested.contains(op.id.as_str()))
        .map(|op| op.id.as_str())
        .collect();
    let missing: Vec<_> = requested.difference(&found).copied().collect();
    if !missing.is_empty() {
        return Err(MigrationDbError::NotFound(format!(
            "operation ids for replay: {}",
            missing.join(", ")
        )));
    }
    Ok(all
        .into_iter()
        .filter(|op| requested.contains(op.id.as_str()))
        .collect())
}

fn catalog_check(checks: &mut Vec<ReplayCheck>, name: &str, recorded: &str, actual: &str) {
    push_check(
        checks,
        name,
        hash_equal(recorded, actual),
        format!("recorded {recorded} recomputed {actual}"),
    );
}

fn push_check(checks: &mut Vec<ReplayCheck>, name: &str, ok: bool, detail: String) {
    checks.push(ReplayCheck {
        name: name.into(),
        status: if ok {
            ValidationStatus::Pass
        } else {
            ValidationStatus::Fail
        },
        detail,
    });
}

fn hash_equal(left: &str, right: &str) -> bool {
    left.strip_prefix("sha256:").unwrap_or(left) == right.strip_prefix("sha256:").unwrap_or(right)
}

fn verify_event_chain(events: &[RunEvent]) -> Result<(bool, String, Option<String>)> {
    let mut previous: Option<String> = None;
    for event in events {
        if event.previous_event_hash != previous {
            return Ok((
                false,
                format!("event {} previous-hash link broken", event.event_seq),
                previous,
            ));
        }
        let mut clean = event.clone();
        clean.event_hash = None;
        let body = serde_json::to_value(&clean)?;
        let material = format!(
            "{}\n{}",
            previous.clone().unwrap_or_default(),
            canonical_json(&body)
        );
        let recomputed = sha256_hex(material.as_bytes());
        if event
            .event_hash
            .as_deref()
            .is_none_or(|recorded| !hash_equal(recorded, &recomputed))
        {
            return Ok((
                false,
                format!("event {} hash mismatch", event.event_seq),
                previous,
            ));
        }
        previous = event.event_hash.clone();
    }
    Ok((true, format!("{} events", events.len()), previous))
}

fn is_blocked_ref(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(part) => {
            let part = part.to_string_lossy().to_ascii_lowercase();
            matches!(part.as_str(), ".env" | "secrets" | "private_keys")
                || part.ends_with(".pem")
                || part.ends_with(".key")
        }
        Component::ParentDir => true,
        _ => false,
    })
}

fn check_file(uri: &str, expected: Option<&str>, root: Option<&Path>) -> Result<FileHashCheck> {
    let mut check = FileHashCheck {
        uri: uri.into(),
        expected_sha256: expected.map(str::to_string),
        actual_sha256: None,
        status: if expected.is_some() {
            "recorded".into()
        } else {
            "missing-hash".into()
        },
        blocked: false,
    };
    let path = Path::new(uri);
    if is_blocked_ref(path) {
        check.status = "blocked-ref".into();
        check.blocked = true;
        return Ok(check);
    }
    let Some(root) = root else {
        return Ok(check);
    };
    let root = root.canonicalize()?;
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    if !candidate.exists() {
        check.status = "missing".into();
        return Ok(check);
    }
    let candidate = candidate.canonicalize()?;
    if !candidate.starts_with(&root) {
        check.status = "out-of-scope".into();
        check.blocked = true;
        return Ok(check);
    }
    if !candidate.is_file() {
        check.status = "missing".into();
        return Ok(check);
    }
    let actual = sha256_hex(&std::fs::read(candidate)?);
    check.actual_sha256 = Some(actual.clone());
    check.status = match expected {
        Some(recorded) if hash_equal(recorded, &actual) => "match",
        Some(_) => "mismatch",
        None => "missing-hash",
    }
    .into();
    Ok(check)
}

fn non_deterministic(operations: &[Operation]) -> Vec<NonDeterministicOperation> {
    operations
        .iter()
        .filter_map(|operation| {
            let marked = operation
                .input_json
                .as_ref()
                .and_then(|value| value.get("non_deterministic"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            (marked
                || NON_DETERMINISTIC_OPERATION_TYPES.contains(&operation.operation_type.as_str()))
            .then(|| NonDeterministicOperation {
                operation_id: operation.id.clone(),
                operation_type: operation.operation_type.clone(),
                risk: operation.risk,
                reason: operation
                    .input_json
                    .as_ref()
                    .and_then(|value| value.get("replay_note"))
                    .and_then(Value::as_str)
                    .unwrap_or("operation requires external or manual state")
                    .into(),
            })
        })
        .collect()
}

fn operation_plan(
    operations: &[Operation],
    checkpoints: &[Checkpoint],
    requested_ids: &[String],
) -> Vec<ReplayOperationPlan> {
    let requested: BTreeSet<&str> = requested_ids.iter().map(String::as_str).collect();
    let mut by_operation: BTreeMap<&str, Vec<Checkpoint>> = BTreeMap::new();
    for checkpoint in checkpoints {
        if let Some(operation_id) = checkpoint.operation_id.as_deref() {
            by_operation
                .entry(operation_id)
                .or_default()
                .push(checkpoint.clone());
        }
    }
    operations
        .iter()
        .filter(|operation| requested.is_empty() || requested.contains(operation.id.as_str()))
        .map(|operation| ReplayOperationPlan {
            operation_id: operation.id.clone(),
            operation_type: operation.operation_type.clone(),
            phase: operation.phase.clone(),
            status: operation.status,
            risk: operation.risk,
            idempotency_key: operation.idempotency_key.clone(),
            command_hash: operation.command_hash.clone(),
            output_ref: operation.output_ref.clone(),
            checkpoint_refs: by_operation
                .remove(operation.id.as_str())
                .unwrap_or_default(),
            replay_action: if operation.status == OpStatus::Succeeded {
                "verify-only"
            } else {
                "resume-from-checkpoint"
            }
            .into(),
        })
        .collect()
}

fn safe_next_action(
    request: &ReplayRequest,
    status: &str,
    mismatches: usize,
    blocked: &[FileHashCheck],
    approvals: &[RequiredApproval],
    non_deterministic: &[NonDeterministicOperation],
) -> String {
    if mismatches > 0 {
        return "stop: inspect hash mismatches before replay".into();
    }
    if !blocked.is_empty() {
        return "stop: remove blocked or out-of-scope replay references".into();
    }
    if !approvals.is_empty() {
        return "request human approval before apply replay".into();
    }
    if !non_deterministic.is_empty() {
        return "dry-run only: manual/non-deterministic operations require operator handling"
            .into();
    }
    if request.mode == ReplayMode::DryRunPlan && matches!(status, "pass" | "partial") {
        return "safe to submit the recorded operation plan to the approval-gated pipeline".into();
    }
    if request.mode == ReplayMode::Apply && status == "pass" {
        return "safe to submit deterministic operations to the normal execution pipeline".into();
    }
    if request.mode == ReplayMode::VerifyOnly && status == "pass" {
        return "replay inputs verified; request a dry-run plan before apply".into();
    }
    "stop: replay result is not clean".into()
}
