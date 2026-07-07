//! The two state machines from DATABASE_FEATURE_SPEC, enforced fail-closed:
//! an unlisted edge is an [`MigrationDbError::IllegalTransition`], and every
//! legal transition is appended to the event ledger by the API layer.

use super::model::{OpStatus, RunStatus};
use super::{MigrationDbError, Result};

/// created -> planning -> awaiting_approval -> running -> paused -> validating -> completed
/// with denied/blocked/failed/cancelled exits per the spec diagram.
pub fn check_run_transition(from: RunStatus, to: RunStatus) -> Result<()> {
    use RunStatus::*;
    let ok = matches!(
        (from, to),
        (Created, Planning)
            | (Created, Cancelled)
            | (Planning, AwaitingApproval)
            | (Planning, Running)
            | (Planning, Blocked)
            | (Planning, Cancelled)
            | (AwaitingApproval, Running)
            | (AwaitingApproval, Denied)
            | (AwaitingApproval, Cancelled)
            | (Running, Paused)
            | (Running, Validating)
            | (Running, Blocked)
            | (Running, Failed)
            | (Running, Cancelled)
            | (Paused, Running)
            | (Paused, Cancelled)
            | (Validating, Completed)
            | (Validating, Failed)
            | (Validating, Running)
            | (Blocked, Running)
            | (Blocked, Cancelled)
    );
    if ok {
        Ok(())
    } else {
        Err(MigrationDbError::IllegalTransition {
            kind: "run",
            from: from.as_str().to_string(),
            to: to.as_str().to_string(),
        })
    }
}

/// queued -> ready -> awaiting_approval -> running -> succeeded, with
/// denied/blocked/failed/cancelled exits per the spec diagram.
pub fn check_op_transition(from: OpStatus, to: OpStatus) -> Result<()> {
    use OpStatus::*;
    let ok = matches!(
        (from, to),
        (Queued, Ready)
            | (Queued, Cancelled)
            | (Ready, AwaitingApproval)
            | (Ready, Running)
            | (Ready, Denied)
            | (Ready, Cancelled)
            | (AwaitingApproval, Ready)
            | (AwaitingApproval, Running)
            | (AwaitingApproval, Denied)
            | (AwaitingApproval, Cancelled)
            | (Running, Succeeded)
            | (Running, Failed)
            | (Running, Blocked)
            | (Running, Cancelled)
            | (Blocked, Ready)
            | (Blocked, Cancelled)
    );
    if ok {
        Ok(())
    } else {
        Err(MigrationDbError::IllegalTransition {
            kind: "operation",
            from: from.as_str().to_string(),
            to: to.as_str().to_string(),
        })
    }
}
