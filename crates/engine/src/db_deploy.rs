//! db_deploy — safe hook/deploy automation (REQ-056).
//!
//! REQ-050 scaffold: the [`DeploySpec`] input, the [`DeployPlan`] /
//! [`DeployStep`] output, and the [`plan`] seam. Behaviour contract
//! (implemented in REQ-056): install through envctl's managed component system
//! into the active layout root, detect running wrappers/hooks and queue/refuse
//! unsafe overwrites, and use atomic temp+rename promotion with rollback.
//! `--apply` never overwrites an executing script in place.

use crate::db::Result;
use serde::{Deserialize, Serialize};

/// What is being deployed and where.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploySpec {
    /// e.g. deploy hooks/wrappers.
    pub kind: String,
    /// Target root, e.g. `$LIFE_OS_ROOT`.
    pub target: String,
    /// When set, the plan is materialized here first for staged promotion.
    pub stage_dir: Option<String>,
}

/// Disposition of a single deploy step — fail-closed toward `Queued`/`Refused`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployDisposition {
    /// Safe to promote atomically.
    Ready,
    /// Target script appears to be executing; promotion is queued.
    Queued,
    /// Refused (protected/never policy or would disturb a running process).
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployStep {
    pub target_path: String,
    pub source_path: String,
    pub disposition: DeployDisposition,
    /// Reason for queue/refuse, for the agent to act on.
    pub reason: String,
    pub rollback_ref: Option<String>,
}

/// The deploy plan the CLI/GUI render and the approval gate reasons over.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployPlan {
    pub steps: Vec<DeployStep>,
    pub ready: usize,
    pub queued: usize,
    pub refused: usize,
    /// True only when apply was confirmed + approved (R3).
    pub approved: bool,
}

/// Build a deploy plan. REQ-056 implements running-process detection + atomic
/// promotion; the scaffold returns an empty plan so nothing can deploy through
/// the seam before the safety logic exists.
pub fn plan(_spec: &DeploySpec, _files: &crate::db_index::FileIndex) -> Result<DeployPlan> {
    Ok(DeployPlan::default())
}
