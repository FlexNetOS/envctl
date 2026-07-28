//! The write API: constraint enforcement (the DDL CHECK/UNIQUE/FK equivalents),
//! the hash-chained append-only event ledger, both state machines wired so every
//! transition appends an event, and the R3+ approval gate (AGENT_CONTROL_PROTOCOL).

use super::machine::{check_op_transition, check_rollback_transition, check_run_transition};
use super::model::*;
use super::store::{self, child_key, event_key};
use super::{canonical_json, now_utc, sha256_hex, MigrationDb, MigrationDbError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSpec {
    pub target_id: String,
    pub target_type: TargetType,
    pub primary_root: String,
    pub compare_root: Option<String>,
    pub descriptor: Value,
    pub safety_mode: String,
    pub max_auto_risk: Risk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSpec {
    pub target_id: String,
    pub recipe_id: String,
    pub human_mode: HumanMode,
    pub initiated_by: Option<String>,
    pub sandbox_policy: Option<String>,
    pub approval_policy: Option<String>,
    pub tool_versions: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSpec {
    pub run_id: String,
    pub operation_type: String,
    pub phase: Option<String>,
    pub risk: Risk,
    /// Explicit idempotency key; when empty it is derived per the protocol:
    /// sha256(run_id + operation_type + target_descriptor_hash + recipe_step_id + input_hash).
    pub idempotency_key: Option<String>,
    pub recipe_step_id: Option<String>,
    pub command_redacted: Option<String>,
    pub input: Option<Value>,
    pub parent_operation_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalDecision {
    Approve,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSpec {
    pub run_id: String,
    pub validator: String,
    pub status: ValidationStatus,
    pub artifact_id: Option<String>,
    pub operation_id: Option<String>,
    pub details: Option<Value>,
    pub evidence: Option<Value>,
}

/// Everything belonging to one run — the `run export` bundle the plugin consumes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunBundle {
    pub run: Run,
    pub target: Target,
    pub recipe: Recipe,
    pub contract: ArtifactContract,
    pub operations: Vec<Operation>,
    pub events: Vec<RunEvent>,
    pub evidence: Vec<Evidence>,
    pub artifacts: Vec<Artifact>,
    pub approvals: Vec<Approval>,
    pub validations: Vec<Validation>,
    pub graph_edges: Vec<GraphEdge>,
    pub checkpoints: Vec<Checkpoint>,
    pub rollbacks: Vec<Rollback>,
}

impl MigrationDb {
    // ---- events -------------------------------------------------------------

    /// Append an event to a run's hash-chained ledger. The hash covers the whole
    /// envelope (with the two hash fields empty) plus the previous event hash, so
    /// any later mutation of history breaks replay verification.
    #[allow(clippy::too_many_arguments)]
    pub fn append_event(
        &self,
        run_id: &str,
        event_type: &str,
        phase: Option<&str>,
        actor_type: ActorType,
        actor_id: Option<&str>,
        operation_id: Option<&str>,
        payload: Value,
        evidence_refs: Option<Value>,
    ) -> Result<RunEvent> {
        // FK: the run must exist (run.created is the one event allowed to self-seed).
        if event_type != "run.created" {
            let _: Run = self.must_get(store::RUNS, run_id, "run")?;
        }
        let seq = self.next_counter(&format!("event_seq:{run_id}"))?;
        let previous = if seq > 1 {
            let prev: RunEvent =
                self.must_get(store::RUN_EVENTS, &event_key(run_id, seq - 1), "event")?;
            prev.event_hash
        } else {
            None
        };
        let mut ev = RunEvent {
            id: format!("{run_id}-ev-{seq:06}"),
            run_id: run_id.to_string(),
            event_seq: seq,
            event_type: event_type.to_string(),
            phase: phase.map(str::to_string),
            actor_type,
            actor_id: actor_id.map(str::to_string),
            operation_id: operation_id.map(str::to_string),
            payload_json: payload,
            evidence_refs_json: evidence_refs,
            previous_event_hash: previous.clone(),
            event_hash: None,
            created_at_utc: now_utc(),
        };
        let body = serde_json::to_value(&ev)?;
        let material = format!(
            "{}\n{}",
            previous.unwrap_or_default(),
            canonical_json(&body)
        );
        ev.event_hash = Some(sha256_hex(material.as_bytes()));
        self.put(store::RUN_EVENTS, &event_key(run_id, seq), &ev, true)?;
        Ok(ev)
    }

    pub fn events(&self, run_id: &str) -> Result<Vec<RunEvent>> {
        self.list(store::RUN_EVENTS, Some(&format!("{run_id}#")))
    }

    // ---- targets ------------------------------------------------------------

    /// Parse + validate + register a target descriptor (UNIQUE target_id).
    pub fn register_target(&self, spec: TargetSpec) -> Result<Target> {
        if spec.target_id.trim().is_empty() {
            return Err(MigrationDbError::Validation("target_id is empty".into()));
        }
        if spec.primary_root.trim().is_empty() {
            return Err(MigrationDbError::Validation("primary_root is empty".into()));
        }
        if !spec.descriptor.is_object() {
            return Err(MigrationDbError::Validation(
                "descriptor must be a JSON object".into(),
            ));
        }
        let existing = match self.target_by_natural_id(&spec.target_id) {
            Ok(target) => Some(target),
            Err(MigrationDbError::NotFound(_)) => None,
            Err(error) => return Err(error),
        };
        let id = match &existing {
            Some(target) => target.id.clone(),
            None => self.next_id("target")?,
        };
        if existing.is_none() {
            self.index_put(store::IDX_TARGET_NATURAL, &spec.target_id, &id, true)?;
        }
        let now = now_utc();
        let target = Target {
            id: id.clone(),
            target_id: spec.target_id,
            target_type: spec.target_type,
            primary_root: spec.primary_root,
            compare_root: spec.compare_root,
            descriptor_hash: sha256_hex(canonical_json(&spec.descriptor).as_bytes()),
            descriptor_json: spec.descriptor,
            safety_mode: spec.safety_mode,
            max_auto_risk: spec.max_auto_risk,
            created_at_utc: existing
                .map(|target| target.created_at_utc)
                .unwrap_or_else(|| now.clone()),
            updated_at_utc: now,
        };
        self.put(store::TARGETS, &id, &target, false)?;
        Ok(target)
    }

    pub fn target_by_natural_id(&self, target_id: &str) -> Result<Target> {
        let id = self
            .index_get(store::IDX_TARGET_NATURAL, target_id)?
            .ok_or_else(|| MigrationDbError::NotFound(format!("target: {target_id}")))?;
        self.must_get(store::TARGETS, &id, "target")
    }

    pub fn targets(&self) -> Result<Vec<Target>> {
        self.list(store::TARGETS, None)
    }

    // ---- packages / contracts / recipes --------------------------------------

    /// Import a package directory: content-hash every file (sorted walk) into a
    /// stable package hash + a small manifest. Read-only on the package.
    pub fn import_package(&self, package_name: &str, package_path: &Path) -> Result<Package> {
        if !package_path.is_dir() {
            return Err(MigrationDbError::Validation(format!(
                "package path is not a directory: {}",
                package_path.display()
            )));
        }
        let mut files: Vec<(String, String, u64)> = Vec::new();
        let mut stack = vec![package_path.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let mut entries: Vec<_> = std::fs::read_dir(&dir)?.collect::<std::io::Result<_>>()?;
            entries.sort_by_key(|e| e.path());
            for entry in entries {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.is_file() {
                    let bytes = std::fs::read(&p)?;
                    let rel = p
                        .strip_prefix(package_path)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .to_string();
                    files.push((rel, sha256_hex(&bytes), bytes.len() as u64));
                }
            }
        }
        files.sort();
        let mut material = String::new();
        let mut total_bytes = 0u64;
        for (rel, hash, len) in &files {
            material.push_str(&format!("{hash}  {rel}\n"));
            total_bytes += len;
        }
        let package_hash = sha256_hex(material.as_bytes());
        let id = self.next_id("package")?;
        let package = Package {
            id: id.clone(),
            package_name: package_name.to_string(),
            package_path: package_path.display().to_string(),
            package_hash,
            manifest_json: json!({
                "file_count": files.len(),
                "total_bytes": total_bytes,
            }),
            imported_at_utc: now_utc(),
        };
        self.put(store::PACKAGES, &id, &package, true)?;
        Ok(package)
    }

    pub fn packages(&self) -> Result<Vec<Package>> {
        self.list(store::PACKAGES, None)
    }

    /// Versioned contract registry (UNIQUE name+version — new versions, not edits).
    pub fn import_contract(
        &self,
        name: &str,
        version: &str,
        contract: Value,
        source_package_id: Option<&str>,
    ) -> Result<ArtifactContract> {
        if let Some(pkg) = source_package_id {
            let _: Package = self.must_get(store::PACKAGES, pkg, "package")?;
        }
        let natural = format!("{name}@{version}");
        let id = self.next_id("contract")?;
        self.index_put(store::IDX_CONTRACT_NATURAL, &natural, &id, true)
            .map_err(|_| {
                MigrationDbError::Conflict(format!(
                    "contract version already exists: {natural} (contract changes create new versions)"
                ))
            })?;
        let row = ArtifactContract {
            id: id.clone(),
            contract_name: name.to_string(),
            contract_version: version.to_string(),
            source_package_id: source_package_id.map(str::to_string),
            contract_hash: sha256_hex(canonical_json(&contract).as_bytes()),
            contract_json: contract,
            created_at_utc: now_utc(),
        };
        self.put(store::CONTRACTS, &id, &row, true)?;
        Ok(row)
    }

    pub fn contracts(&self) -> Result<Vec<ArtifactContract>> {
        self.list(store::CONTRACTS, None)
    }

    pub fn create_recipe(
        &self,
        name: &str,
        version: &str,
        artifact_contract_id: &str,
        recipe: Value,
    ) -> Result<Recipe> {
        let _: ArtifactContract =
            self.must_get(store::CONTRACTS, artifact_contract_id, "artifact contract")?;
        if !recipe.get("steps").map(Value::is_array).unwrap_or(false) {
            return Err(MigrationDbError::Validation(
                "recipe must contain a steps array".into(),
            ));
        }
        let natural = format!("{name}@{version}");
        let id = self.next_id("recipe")?;
        self.index_put(store::IDX_RECIPE_NATURAL, &natural, &id, true)
            .map_err(|_| {
                MigrationDbError::Conflict(format!("recipe version already exists: {natural}"))
            })?;
        let row = Recipe {
            id: id.clone(),
            recipe_name: name.to_string(),
            recipe_version: version.to_string(),
            artifact_contract_id: artifact_contract_id.to_string(),
            recipe_hash: sha256_hex(canonical_json(&recipe).as_bytes()),
            recipe_json: recipe,
            created_at_utc: now_utc(),
        };
        self.put(store::RECIPES, &id, &row, true)?;
        Ok(row)
    }

    pub fn recipes(&self) -> Result<Vec<Recipe>> {
        self.list(store::RECIPES, None)
    }

    // ---- runs -----------------------------------------------------------------

    pub fn create_run(&self, spec: RunSpec, actor: ActorType, actor_id: &str) -> Result<Run> {
        let target: Target = self.must_get(store::TARGETS, &spec.target_id, "target")?;
        let recipe: Recipe = self.must_get(store::RECIPES, &spec.recipe_id, "recipe")?;
        let id = self.next_id("run")?;
        let run = Run {
            id: id.clone(),
            target_id: target.id.clone(),
            recipe_id: recipe.id.clone(),
            artifact_contract_id: recipe.artifact_contract_id.clone(),
            status: RunStatus::Created,
            human_mode: spec.human_mode,
            initiated_by: spec.initiated_by,
            sandbox_policy: spec.sandbox_policy,
            approval_policy: spec.approval_policy,
            tool_versions_json: spec.tool_versions,
            reproducibility_hash: None,
            started_at_utc: None,
            completed_at_utc: None,
            created_at_utc: now_utc(),
        };
        self.put(store::RUNS, &id, &run, true)?;
        self.append_event(
            &id,
            "run.created",
            None,
            actor,
            Some(actor_id),
            None,
            json!({
                "target_id": target.target_id,
                "target_hash": target.descriptor_hash,
                "recipe": format!("{}@{}", recipe.recipe_name, recipe.recipe_version),
                "recipe_hash": recipe.recipe_hash,
                "human_mode": run.human_mode.as_str(),
            }),
            None,
        )?;
        Ok(run)
    }

    pub fn run(&self, run_id: &str) -> Result<Run> {
        self.must_get(store::RUNS, run_id, "run")
    }

    pub fn runs(&self) -> Result<Vec<Run>> {
        self.list(store::RUNS, None)
    }

    /// Transition a run through the state machine; the transition is the event.
    pub fn run_set_status(
        &self,
        run_id: &str,
        to: RunStatus,
        actor: ActorType,
        actor_id: &str,
        reason: Option<&str>,
    ) -> Result<Run> {
        let mut run = self.run(run_id)?;
        check_run_transition(run.status, to)?;
        let from = run.status;
        run.status = to;
        let now = now_utc();
        if to == RunStatus::Running && run.started_at_utc.is_none() {
            run.started_at_utc = Some(now.clone());
        }
        if matches!(
            to,
            RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled | RunStatus::Denied
        ) {
            run.completed_at_utc = Some(now.clone());
        }
        self.put(store::RUNS, run_id, &run, false)?;
        self.append_event(
            run_id,
            "run.status_changed",
            None,
            actor,
            Some(actor_id),
            None,
            json!({"from": from.as_str(), "to": to.as_str(), "reason": reason}),
            None,
        )?;
        Ok(run)
    }

    /// Completing a run stamps its reproducibility hash over the whole recorded
    /// identity: target + recipe + contract + tool versions + final event hash.
    pub fn complete_run(&self, run_id: &str, actor: ActorType, actor_id: &str) -> Result<Run> {
        let run = self.run(run_id)?;
        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?;
        let contract: ArtifactContract =
            self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?;
        // Transition FIRST so the hash covers the completion event itself —
        // replay recomputes over the full chain and must land on this value.
        let mut run = self.run_set_status(run_id, RunStatus::Completed, actor, actor_id, None)?;
        let events = self.events(run_id)?;
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
        run.reproducibility_hash = Some(sha256_hex(material.as_bytes()));
        self.put(store::RUNS, run_id, &run, false)?;
        Ok(run)
    }

    // ---- operations -----------------------------------------------------------

    /// Queue an operation. Idempotent: an existing (run, idempotency_key) pair
    /// returns the already-recorded operation instead of a duplicate.
    pub fn add_operation(
        &self,
        spec: OperationSpec,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Operation> {
        let run = self.run(&spec.run_id)?;
        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let input_json = spec.input.clone().unwrap_or(Value::Null);
        let idem = match &spec.idempotency_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => sha256_hex(
                format!(
                    "{}{}{}{}{}",
                    spec.run_id,
                    spec.operation_type,
                    target.descriptor_hash,
                    spec.recipe_step_id.clone().unwrap_or_default(),
                    sha256_hex(canonical_json(&input_json).as_bytes()),
                )
                .as_bytes(),
            ),
        };
        let idem_key = child_key(&spec.run_id, &idem);
        if let Some(existing) = self.index_get(store::IDX_OP_IDEMPOTENCY, &idem_key)? {
            return self.must_get(store::OPERATIONS, &existing, "operation");
        }
        let id = self.next_id("op")?;
        let op = Operation {
            id: id.clone(),
            run_id: spec.run_id.clone(),
            parent_operation_id: spec.parent_operation_id,
            operation_type: spec.operation_type.clone(),
            phase: spec.phase.clone(),
            status: OpStatus::Queued,
            risk: spec.risk,
            idempotency_key: idem,
            command_hash: spec
                .command_redacted
                .as_ref()
                .map(|c| sha256_hex(c.as_bytes())),
            command_redacted: spec.command_redacted,
            input_json: spec.input,
            output_ref: None,
            error_json: None,
            started_at_utc: None,
            completed_at_utc: None,
            created_at_utc: now_utc(),
        };
        self.put(store::OPERATIONS, &id, &op, true)?;
        self.index_put(
            store::IDX_OPS_BY_RUN,
            &child_key(&spec.run_id, &id),
            &id,
            true,
        )?;
        self.index_put(store::IDX_OP_IDEMPOTENCY, &idem_key, &id, true)?;
        self.append_event(
            &spec.run_id,
            "operation.queued",
            spec.phase.as_deref(),
            actor,
            Some(actor_id),
            Some(&id),
            json!({"operation_type": op.operation_type, "risk": op.risk.as_str()}),
            None,
        )?;
        Ok(op)
    }

    pub fn operation(&self, op_id: &str) -> Result<Operation> {
        self.must_get(store::OPERATIONS, op_id, "operation")
    }

    pub fn operations(&self, run_id: &str) -> Result<Vec<Operation>> {
        let ids = self.index_children(store::IDX_OPS_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::OPERATIONS, id, "operation"))
            .collect()
    }

    pub fn op_set_status(
        &self,
        op_id: &str,
        to: OpStatus,
        actor: ActorType,
        actor_id: &str,
        detail: Option<Value>,
    ) -> Result<Operation> {
        let mut op = self.operation(op_id)?;
        check_op_transition(op.status, to)?;
        let from = op.status;
        op.status = to;
        let now = now_utc();
        if to == OpStatus::Running && op.started_at_utc.is_none() {
            op.started_at_utc = Some(now.clone());
        }
        if matches!(
            to,
            OpStatus::Succeeded | OpStatus::Failed | OpStatus::Denied | OpStatus::Cancelled
        ) {
            op.completed_at_utc = Some(now.clone());
        }
        if to == OpStatus::Failed {
            op.error_json = detail.clone();
        }
        self.put(store::OPERATIONS, op_id, &op, false)?;
        self.append_event(
            &op.run_id,
            "operation.status_changed",
            op.phase.as_deref(),
            actor,
            Some(actor_id),
            Some(op_id),
            json!({"from": from.as_str(), "to": to.as_str(), "detail": detail}),
            None,
        )?;
        Ok(op)
    }

    /// The approval gate. R3+ (or anything above the target's max_auto_risk)
    /// cannot start without an approved approval: the operation parks in
    /// awaiting_approval with an open approval row. Safe risks start directly.
    pub fn op_request_start(
        &self,
        op_id: &str,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<(Operation, Option<Approval>)> {
        let op = self.operation(op_id)?;
        let run = self.run(&op.run_id)?;
        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let needs_approval =
            op.risk.requires_approval() || (op.risk.as_str() > target.max_auto_risk.as_str());
        let approved_already = self
            .approvals(&op.run_id)?
            .into_iter()
            .any(|a| a.operation_id == op_id && a.status == ApprovalStatus::Approved);
        if op.status == OpStatus::Queued {
            self.op_set_status(op_id, OpStatus::Ready, actor, actor_id, None)?;
        }
        if needs_approval && !approved_already {
            let approval = self.request_approval(op_id, actor, actor_id)?;
            let op =
                self.op_set_status(op_id, OpStatus::AwaitingApproval, actor, actor_id, None)?;
            return Ok((op, Some(approval)));
        }
        let op = self.op_set_status(op_id, OpStatus::Running, actor, actor_id, None)?;
        Ok((op, None))
    }

    // ---- approvals --------------------------------------------------------------

    pub fn request_approval(
        &self,
        op_id: &str,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Approval> {
        let op = self.operation(op_id)?;
        let id = self.next_id("approval")?;
        let approval = Approval {
            id: id.clone(),
            run_id: op.run_id.clone(),
            operation_id: op_id.to_string(),
            risk: op.risk,
            status: ApprovalStatus::Open,
            requested_by: Some(actor_id.to_string()),
            decided_by: None,
            reason: None,
            requested_at_utc: now_utc(),
            decided_at_utc: None,
        };
        self.put(store::APPROVALS, &id, &approval, true)?;
        self.index_put(
            store::IDX_APPROVALS_BY_RUN,
            &child_key(&op.run_id, &id),
            &id,
            true,
        )?;
        self.append_event(
            &op.run_id,
            "approval.requested",
            op.phase.as_deref(),
            actor,
            Some(actor_id),
            Some(op_id),
            json!({"approval_id": id, "risk": op.risk.as_str()}),
            None,
        )?;
        Ok(approval)
    }

    /// Decide an approval. The decision, the decider, the rationale, and the
    /// evidence refs all land in the ledger — agent reviewers use exactly the
    /// same surface as humans (authority, not state, is the difference).
    pub fn approval_decide(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
        actor: ActorType,
        decided_by: &str,
        reason: &str,
        evidence_refs: Option<Value>,
    ) -> Result<Approval> {
        let mut approval: Approval = self.must_get(store::APPROVALS, approval_id, "approval")?;
        if approval.status != ApprovalStatus::Open {
            return Err(MigrationDbError::Conflict(format!(
                "approval {approval_id} already {}",
                approval.status.as_str()
            )));
        }
        approval.status = match decision {
            ApprovalDecision::Approve => ApprovalStatus::Approved,
            ApprovalDecision::Deny => ApprovalStatus::Denied,
        };
        approval.decided_by = Some(decided_by.to_string());
        approval.reason = Some(reason.to_string());
        approval.decided_at_utc = Some(now_utc());
        self.put(store::APPROVALS, approval_id, &approval, false)?;
        self.append_event(
            &approval.run_id,
            "approval.decided",
            None,
            actor,
            Some(decided_by),
            Some(&approval.operation_id),
            json!({
                "approval_id": approval_id,
                "decision": approval.status.as_str(),
                "reason": reason,
            }),
            evidence_refs,
        )?;
        // A rollback can request a fresh approval for an operation that has
        // already succeeded.  In that case the decision authorizes the
        // rollback handle, not a new operation transition.
        if self.operation(&approval.operation_id)?.status == OpStatus::AwaitingApproval {
            let next = match decision {
                ApprovalDecision::Approve => OpStatus::Ready,
                ApprovalDecision::Deny => OpStatus::Denied,
            };
            self.op_set_status(&approval.operation_id, next, actor, decided_by, None)?;
        }
        Ok(approval)
    }

    pub fn approvals(&self, run_id: &str) -> Result<Vec<Approval>> {
        let ids = self.index_children(store::IDX_APPROVALS_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::APPROVALS, id, "approval"))
            .collect()
    }

    pub fn all_approvals(&self) -> Result<Vec<Approval>> {
        self.list(store::APPROVALS, None)
    }

    // ---- evidence / artifacts / validations / edges ------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn add_evidence(
        &self,
        run_id: &str,
        operation_id: Option<&str>,
        uri: &str,
        kind: &str,
        sha256: Option<&str>,
        redacted: bool,
        metadata: Option<Value>,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Evidence> {
        let _ = self.run(run_id)?;
        let id = self.next_id("evidence")?;
        let ev = Evidence {
            id: id.clone(),
            run_id: run_id.to_string(),
            operation_id: operation_id.map(str::to_string),
            uri: uri.to_string(),
            evidence_kind: kind.to_string(),
            sha256: sha256.map(str::to_string),
            redacted,
            metadata_json: metadata,
            created_at_utc: now_utc(),
        };
        self.put(store::EVIDENCE, &id, &ev, true)?;
        self.index_put(
            store::IDX_EVIDENCE_BY_RUN,
            &child_key(run_id, &id),
            &id,
            true,
        )?;
        self.append_event(
            run_id,
            "evidence.recorded",
            None,
            actor,
            Some(actor_id),
            operation_id,
            json!({"evidence_id": id, "uri": uri, "kind": kind, "sha256": sha256}),
            None,
        )?;
        Ok(ev)
    }

    pub fn evidence(&self, run_id: &str) -> Result<Vec<Evidence>> {
        let ids = self.index_children(store::IDX_EVIDENCE_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::EVIDENCE, id, "evidence"))
            .collect()
    }

    /// Upsert an artifact record (UNIQUE(run_id, artifact_id) — updates refresh).
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_artifact(
        &self,
        run_id: &str,
        artifact_id: &str,
        title: &str,
        artifact_type: Option<&str>,
        status: ArtifactStatus,
        path: Option<&str>,
        content_hash: Option<&str>,
        generated_by_operation_id: Option<&str>,
        evidence: Option<Value>,
        links: Option<Value>,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Artifact> {
        let _ = self.run(run_id)?;
        let key = child_key(run_id, artifact_id);
        let now = now_utc();
        let existing: Option<Artifact> = self.get(store::ARTIFACTS, &key)?;
        let row = Artifact {
            id: existing
                .as_ref()
                .map(|a| a.id.clone())
                .unwrap_or(self.next_id("artifact")?),
            run_id: run_id.to_string(),
            artifact_id: artifact_id.to_string(),
            title: title.to_string(),
            artifact_type: artifact_type.map(str::to_string),
            status,
            path: path.map(str::to_string),
            content_hash: content_hash.map(str::to_string),
            generated_by_operation_id: generated_by_operation_id.map(str::to_string),
            evidence_json: evidence,
            links_json: links,
            created_at_utc: existing
                .as_ref()
                .map(|a| a.created_at_utc.clone())
                .unwrap_or(now.clone()),
            updated_at_utc: now,
        };
        let is_new = existing.is_none();
        self.put(store::ARTIFACTS, &key, &row, false)?;
        if is_new {
            self.index_put(store::IDX_ARTIFACTS_BY_RUN, &key, &key, true)?;
        }
        self.append_event(
            run_id,
            if is_new { "artifact.recorded" } else { "artifact.updated" },
            None,
            actor,
            Some(actor_id),
            generated_by_operation_id,
            json!({"artifact_id": artifact_id, "status": row.status.as_str(), "content_hash": content_hash}),
            None,
        )?;
        Ok(row)
    }

    pub fn artifacts(&self, run_id: &str) -> Result<Vec<Artifact>> {
        self.list(store::ARTIFACTS, Some(&format!("{run_id}#")))
    }

    pub fn add_validation(
        &self,
        spec: ValidationSpec,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Validation> {
        let _ = self.run(&spec.run_id)?;
        let id = self.next_id("validation")?;
        let row = Validation {
            id: id.clone(),
            run_id: spec.run_id.clone(),
            artifact_id: spec.artifact_id,
            operation_id: spec.operation_id.clone(),
            validator: spec.validator.clone(),
            status: spec.status,
            details_json: spec.details,
            evidence_json: spec.evidence,
            created_at_utc: now_utc(),
        };
        self.put(store::VALIDATIONS, &id, &row, true)?;
        self.index_put(
            store::IDX_VALIDATIONS_BY_RUN,
            &child_key(&spec.run_id, &id),
            &id,
            true,
        )?;
        self.append_event(
            &spec.run_id,
            "validation.recorded",
            None,
            actor,
            Some(actor_id),
            spec.operation_id.as_deref(),
            json!({"validator": spec.validator, "status": spec.status.as_str()}),
            None,
        )?;
        Ok(row)
    }

    pub fn validations(&self, run_id: &str) -> Result<Vec<Validation>> {
        let ids = self.index_children(store::IDX_VALIDATIONS_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::VALIDATIONS, id, "validation"))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_graph_edge(
        &self,
        run_id: &str,
        from_node: &str,
        to_node: &str,
        edge_type: &str,
        source_artifact_id: Option<&str>,
        confidence: Option<&str>,
        evidence: Option<Value>,
    ) -> Result<GraphEdge> {
        let _ = self.run(run_id)?;
        let id = self.next_id("edge")?;
        let row = GraphEdge {
            id: id.clone(),
            run_id: run_id.to_string(),
            from_node: from_node.to_string(),
            to_node: to_node.to_string(),
            edge_type: edge_type.to_string(),
            source_artifact_id: source_artifact_id.map(str::to_string),
            confidence: confidence.map(str::to_string),
            evidence_json: evidence,
            created_at_utc: now_utc(),
        };
        self.put(store::GRAPH_EDGES, &id, &row, true)?;
        self.index_put(store::IDX_EDGES_BY_RUN, &child_key(run_id, &id), &id, true)?;
        Ok(row)
    }

    pub fn graph_edges(&self, run_id: &str) -> Result<Vec<GraphEdge>> {
        let ids = self.index_children(store::IDX_EDGES_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::GRAPH_EDGES, id, "graph edge"))
            .collect()
    }

    // ---- checkpoints / rollbacks / sessions --------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn add_checkpoint(
        &self,
        run_id: &str,
        operation_id: Option<&str>,
        kind: &str,
        reference: &str,
        hash: Option<&str>,
        metadata: Option<Value>,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Checkpoint> {
        let _ = self.run(run_id)?;
        if kind.trim().is_empty() || reference.trim().is_empty() {
            return Err(MigrationDbError::Validation(
                "checkpoint kind and reference must be non-empty".into(),
            ));
        }
        if reference.split(['/', '\\']).any(|part| part == "..")
            || reference.split(['/', '\\']).any(|part| {
                matches!(
                    part.to_ascii_lowercase().as_str(),
                    "secrets" | "private_keys"
                )
            })
            || [".pem", ".key"]
                .iter()
                .any(|suffix| reference.to_ascii_lowercase().ends_with(suffix))
        {
            return Err(MigrationDbError::Validation(
                "checkpoint reference is not an approved non-secret location".into(),
            ));
        }
        if let Some(operation_id) = operation_id {
            let operation = self.operation(operation_id)?;
            if operation.run_id != run_id {
                return Err(MigrationDbError::Validation(format!(
                    "operation {operation_id} does not belong to run {run_id}"
                )));
            }
        }
        let checkpoint_hash = hash.map(str::to_string).unwrap_or_else(|| {
            sha256_hex(
                canonical_json(&json!({
                    "kind": kind,
                    "reference": reference,
                    "metadata": metadata,
                }))
                .as_bytes(),
            )
        });
        if let Some(existing) = self.checkpoints(run_id)?.into_iter().find(|checkpoint| {
            checkpoint.operation_id.as_deref() == operation_id
                && checkpoint.checkpoint_kind == kind
                && checkpoint.checkpoint_ref == reference
                && checkpoint.checkpoint_hash.as_deref() == Some(checkpoint_hash.as_str())
        }) {
            return Ok(existing);
        }
        let id = self.next_id("checkpoint")?;
        let row = Checkpoint {
            id: id.clone(),
            run_id: run_id.to_string(),
            operation_id: operation_id.map(str::to_string),
            checkpoint_kind: kind.to_string(),
            checkpoint_ref: reference.to_string(),
            checkpoint_hash: Some(checkpoint_hash),
            metadata_json: metadata,
            created_at_utc: now_utc(),
        };
        self.put(store::CHECKPOINTS, &id, &row, true)?;
        self.index_put(
            store::IDX_CHECKPOINTS_BY_RUN,
            &child_key(run_id, &id),
            &id,
            true,
        )?;
        self.append_event(
            run_id,
            "checkpoint.recorded",
            None,
            actor,
            Some(actor_id),
            operation_id,
            json!({"checkpoint_id": id, "kind": kind, "ref": reference}),
            None,
        )?;
        Ok(row)
    }

    pub fn checkpoints(&self, run_id: &str) -> Result<Vec<Checkpoint>> {
        let ids = self.index_children(store::IDX_CHECKPOINTS_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::CHECKPOINTS, id, "checkpoint"))
            .collect()
    }

    pub fn plan_rollback(
        &self,
        run_id: &str,
        operation_id: Option<&str>,
        rollback_type: &str,
        plan: Value,
        actor: ActorType,
        actor_id: &str,
    ) -> Result<Rollback> {
        let _ = self.run(run_id)?;
        if rollback_type.trim().is_empty() || !plan.is_object() {
            return Err(MigrationDbError::Validation(
                "rollback type must be non-empty and plan must be a JSON object".into(),
            ));
        }
        let approval_required = if let Some(operation_id) = operation_id {
            let operation = self.operation(operation_id)?;
            if operation.run_id != run_id {
                return Err(MigrationDbError::Validation(format!(
                    "operation {operation_id} does not belong to run {run_id}"
                )));
            }
            operation.risk.requires_approval()
        } else {
            false
        };
        let id = self.next_id("rollback")?;
        let row = Rollback {
            id: id.clone(),
            run_id: run_id.to_string(),
            operation_id: operation_id.map(str::to_string),
            rollback_type: rollback_type.to_string(),
            status: if approval_required {
                RollbackStatus::AwaitingApproval
            } else {
                RollbackStatus::Planned
            },
            plan_json: plan,
            result_json: None,
            created_at_utc: now_utc(),
        };
        self.put(store::ROLLBACKS, &id, &row, true)?;
        self.index_put(
            store::IDX_ROLLBACKS_BY_RUN,
            &child_key(run_id, &id),
            &id,
            true,
        )?;
        self.append_event(
            run_id,
            "rollback.planned",
            None,
            actor,
            Some(actor_id),
            operation_id,
            json!({"rollback_id": id, "type": rollback_type}),
            None,
        )?;
        if approval_required {
            // Approval rows are already scoped to run + operation and are
            // hash-ledgered.  No operation status is changed here: this is an
            // authorization for the rollback handle, not a replay of work.
            self.request_approval(operation_id.expect("checked above"), actor, actor_id)?;
        }
        Ok(row)
    }

    pub fn rollbacks(&self, run_id: &str) -> Result<Vec<Rollback>> {
        let ids = self.index_children(store::IDX_ROLLBACKS_BY_RUN, run_id)?;
        ids.iter()
            .map(|id| self.must_get(store::ROLLBACKS, id, "rollback"))
            .collect()
    }

    /// Move a rollback handle through its fail-closed lifecycle.  The only
    /// route out of `awaiting_approval` requires an approved decision for the
    /// linked operation; terminal handles cannot be replayed accidentally.
    pub fn rollback_set_status(
        &self,
        rollback_id: &str,
        to: RollbackStatus,
        actor: ActorType,
        actor_id: &str,
        result: Option<Value>,
    ) -> Result<Rollback> {
        let mut rollback: Rollback = self.must_get(store::ROLLBACKS, rollback_id, "rollback")?;
        check_rollback_transition(rollback.status, to)?;
        if rollback.status == RollbackStatus::AwaitingApproval && to == RollbackStatus::Planned {
            let operation_id = rollback.operation_id.as_deref().ok_or_else(|| {
                MigrationDbError::ApprovalRequired(format!(
                    "rollback {rollback_id} has no operation approval scope"
                ))
            })?;
            let approved = self
                .approvals(&rollback.run_id)?
                .into_iter()
                .any(|approval| {
                    approval.operation_id == operation_id
                        && approval.status == ApprovalStatus::Approved
                });
            if !approved {
                return Err(MigrationDbError::ApprovalRequired(format!(
                    "rollback {rollback_id} requires an approved decision"
                )));
            }
        }
        let from = rollback.status;
        rollback.status = to;
        if result.is_some() {
            rollback.result_json = result.clone();
        }
        self.put(store::ROLLBACKS, rollback_id, &rollback, false)?;
        self.append_event(
            &rollback.run_id,
            "rollback.status_changed",
            None,
            actor,
            Some(actor_id),
            rollback.operation_id.as_deref(),
            json!({"rollback_id": rollback_id, "from": from.as_str(), "to": to.as_str(), "result": result}),
            None,
        )?;
        Ok(rollback)
    }

    pub fn record_agent_session(
        &self,
        run_id: Option<&str>,
        agent_name: &str,
        model_label: Option<&str>,
        authority_level: Option<&str>,
        session: Option<Value>,
    ) -> Result<AgentSession> {
        if let Some(r) = run_id {
            let _ = self.run(r)?;
        }
        let id = self.next_id("agent-session")?;
        let row = AgentSession {
            id: id.clone(),
            run_id: run_id.map(str::to_string),
            agent_name: agent_name.to_string(),
            model_label: model_label.map(str::to_string),
            authority_level: authority_level.map(str::to_string),
            session_json: session,
            created_at_utc: now_utc(),
        };
        self.put(store::AGENT_SESSIONS, &id, &row, true)?;
        Ok(row)
    }

    pub fn agent_sessions(&self) -> Result<Vec<AgentSession>> {
        self.list(store::AGENT_SESSIONS, None)
    }

    pub fn record_plugin_session(
        &self,
        run_id: Option<&str>,
        plugin_name: &str,
        plugin_version: Option<&str>,
        nu_version: Option<&str>,
        human_mode: Option<HumanMode>,
        session: Option<Value>,
    ) -> Result<PluginSession> {
        if let Some(r) = run_id {
            let _ = self.run(r)?;
        }
        let id = self.next_id("plugin-session")?;
        let row = PluginSession {
            id: id.clone(),
            run_id: run_id.map(str::to_string),
            plugin_name: plugin_name.to_string(),
            plugin_version: plugin_version.map(str::to_string),
            nu_version: nu_version.map(str::to_string),
            human_mode,
            session_json: session,
            created_at_utc: now_utc(),
        };
        self.put(store::PLUGIN_SESSIONS, &id, &row, true)?;
        Ok(row)
    }

    // ---- export --------------------------------------------------------------

    /// The whole run as one bundle (the plugin/agent read surface).
    pub fn export_run(&self, run_id: &str) -> Result<RunBundle> {
        let run = self.run(run_id)?;
        Ok(RunBundle {
            target: self.must_get(store::TARGETS, &run.target_id, "target")?,
            recipe: self.must_get(store::RECIPES, &run.recipe_id, "recipe")?,
            contract: self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?,
            operations: self.operations(run_id)?,
            events: self.events(run_id)?,
            evidence: self.evidence(run_id)?,
            artifacts: self.artifacts(run_id)?,
            approvals: self.approvals(run_id)?,
            validations: self.validations(run_id)?,
            graph_edges: self.graph_edges(run_id)?,
            checkpoints: self.checkpoints(run_id)?,
            rollbacks: self.rollbacks(run_id)?,
            run,
        })
    }
}
