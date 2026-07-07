//! The sql/002 query views as read functions. Row shapes mirror the SQL view
//! columns so the CLI/plugin wire format matches the package contract.

use super::model::*;
use super::store;
use super::{MigrationDb, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// envctl_migration_run_latest_status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStatusRow {
    pub run_id: String,
    pub target_id: String,
    pub target_type: TargetType,
    pub status: RunStatus,
    pub human_mode: HumanMode,
    pub started_at_utc: Option<String>,
    pub completed_at_utc: Option<String>,
    pub operation_count: usize,
    pub failed_operation_count: usize,
    pub open_approval_count: usize,
    pub artifact_count: usize,
    pub last_event_at_utc: Option<String>,
}

/// envctl_migration_live_timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineRow {
    pub run_id: String,
    pub event_seq: u64,
    pub created_at_utc: String,
    pub phase: Option<String>,
    pub event_type: String,
    pub actor_type: ActorType,
    pub actor_id: Option<String>,
    pub operation_id: Option<String>,
    pub operation_type: Option<String>,
    pub operation_status: Option<OpStatus>,
    pub payload_json: Value,
}

/// envctl_migration_open_approvals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRow {
    pub approval_id: String,
    pub run_id: String,
    pub operation_id: String,
    pub operation_type: String,
    pub risk: Risk,
    pub status: ApprovalStatus,
    pub requested_by: Option<String>,
    pub reason: Option<String>,
    pub requested_at_utc: String,
}

/// envctl_migration_validation_scorecard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardRow {
    pub run_id: String,
    pub pass_count: usize,
    pub fail_count: usize,
    pub warn_count: usize,
    pub blocked_count: usize,
    pub unknown_count: usize,
}

/// envctl_migration_replay_readiness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayReadinessRow {
    pub run_id: String,
    pub status: RunStatus,
    pub reproducibility_hash: Option<String>,
    pub has_reproducibility_hash: bool,
    pub evidence_missing_hashes: usize,
    pub artifacts_missing_hashes: usize,
    pub open_approvals: usize,
}

impl MigrationDb {
    pub fn view_run_status(&self, run_id: &str) -> Result<RunStatusRow> {
        let run = self.run(run_id)?;
        let target: Target = self.must_get(store::TARGETS, &run.target_id, "target")?;
        let ops = self.operations(run_id)?;
        let approvals = self.approvals(run_id)?;
        let artifacts = self.artifacts(run_id)?;
        let events = self.events(run_id)?;
        Ok(RunStatusRow {
            run_id: run.id,
            target_id: target.target_id,
            target_type: target.target_type,
            status: run.status,
            human_mode: run.human_mode,
            started_at_utc: run.started_at_utc,
            completed_at_utc: run.completed_at_utc,
            operation_count: ops.len(),
            failed_operation_count: ops.iter().filter(|o| o.status == OpStatus::Failed).count(),
            open_approval_count: approvals
                .iter()
                .filter(|a| a.status == ApprovalStatus::Open)
                .count(),
            artifact_count: artifacts.len(),
            last_event_at_utc: events.last().map(|e| e.created_at_utc.clone()),
        })
    }

    pub fn view_all_run_status(&self) -> Result<Vec<RunStatusRow>> {
        self.runs()?
            .iter()
            .map(|r| self.view_run_status(&r.id))
            .collect()
    }

    pub fn view_timeline(&self, run_id: &str) -> Result<Vec<TimelineRow>> {
        let events = self.events(run_id)?;
        events
            .into_iter()
            .map(|e| {
                let op = match &e.operation_id {
                    Some(op_id) => self.get::<Operation>(store::OPERATIONS, op_id)?,
                    None => None,
                };
                Ok(TimelineRow {
                    run_id: e.run_id,
                    event_seq: e.event_seq,
                    created_at_utc: e.created_at_utc,
                    phase: e.phase,
                    event_type: e.event_type,
                    actor_type: e.actor_type,
                    actor_id: e.actor_id,
                    operation_id: e.operation_id,
                    operation_type: op.as_ref().map(|o| o.operation_type.clone()),
                    operation_status: op.as_ref().map(|o| o.status),
                    payload_json: e.payload_json,
                })
            })
            .collect()
    }

    /// The live operation queue: everything not yet terminal.
    pub fn view_operation_queue(&self, run_id: &str) -> Result<Vec<Operation>> {
        Ok(self
            .operations(run_id)?
            .into_iter()
            .filter(|o| {
                matches!(
                    o.status,
                    OpStatus::Queued
                        | OpStatus::Ready
                        | OpStatus::AwaitingApproval
                        | OpStatus::Running
                        | OpStatus::Blocked
                )
            })
            .collect())
    }

    pub fn view_open_approvals(&self, run_id: Option<&str>) -> Result<Vec<ApprovalRow>> {
        let approvals = match run_id {
            Some(r) => self.approvals(r)?,
            None => self.all_approvals()?,
        };
        approvals
            .into_iter()
            .filter(|a| a.status == ApprovalStatus::Open)
            .map(|a| {
                let op = self.operation(&a.operation_id)?;
                Ok(ApprovalRow {
                    approval_id: a.id,
                    run_id: a.run_id,
                    operation_id: a.operation_id,
                    operation_type: op.operation_type,
                    risk: a.risk,
                    status: a.status,
                    requested_by: a.requested_by,
                    reason: a.reason,
                    requested_at_utc: a.requested_at_utc,
                })
            })
            .collect()
    }

    pub fn view_scorecard(&self, run_id: &str) -> Result<ScorecardRow> {
        let vals = self.validations(run_id)?;
        let count = |s: ValidationStatus| vals.iter().filter(|v| v.status == s).count();
        Ok(ScorecardRow {
            run_id: run_id.to_string(),
            pass_count: count(ValidationStatus::Pass),
            fail_count: count(ValidationStatus::Fail),
            warn_count: count(ValidationStatus::Warn),
            blocked_count: count(ValidationStatus::Blocked),
            unknown_count: count(ValidationStatus::Unknown),
        })
    }

    pub fn view_replay_readiness(&self, run_id: &str) -> Result<ReplayReadinessRow> {
        let run = self.run(run_id)?;
        let evidence = self.evidence(run_id)?;
        let artifacts = self.artifacts(run_id)?;
        let approvals = self.approvals(run_id)?;
        Ok(ReplayReadinessRow {
            run_id: run.id,
            status: run.status,
            has_reproducibility_hash: run.reproducibility_hash.is_some(),
            reproducibility_hash: run.reproducibility_hash,
            evidence_missing_hashes: evidence.iter().filter(|e| e.sha256.is_none()).count(),
            artifacts_missing_hashes: artifacts
                .iter()
                .filter(|a| a.content_hash.is_none())
                .count(),
            open_approvals: approvals
                .iter()
                .filter(|a| a.status == ApprovalStatus::Open)
                .count(),
        })
    }
}
