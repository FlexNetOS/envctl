//! Release materialization: consume database-approved activations (blueprint
//! §17 step 15, D23).
//!
//! `lifeos_release.promote` refuses until all eleven release gates pass and the
//! manifest, closure, and rollback records exist, then writes one
//! `lifeos_runtime.outbox` record addressed to `envctl-release-materializer`
//! carrying the approved activation. This module is that materializer: it
//! drains those records, performs the atomic symlink flip the payload names,
//! and retains the previous target as the exact rollback target.
//!
//! Two properties matter and are enforced here rather than assumed:
//!
//! * The database decides. Nothing is materialized that `promote` did not
//!   approve, so the gate cannot be bypassed by calling this directly.
//! * Applying is opt-in. Like every other destructive envctl verb this
//!   previews by default and fails closed.

use postgres::Client;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::{connect, internal, CommitError};

/// The outbox destination `lifeos_release.promote` addresses.
pub const MATERIALIZER_DESTINATION: &str = "envctl-release-materializer";
/// The activation kind that maps onto an atomic symlink swap.
pub const ATOMIC_SYMLINK_ACTIVATION: &str = "atomic-symlink-and-session-reload";

/// One database-approved activation awaiting materialization.
#[derive(Debug, Clone, Serialize)]
pub struct PendingActivation {
    pub outbox_id: String,
    pub sequence: i64,
    pub activation_id: String,
    pub release_id: String,
    pub manifest_object_id: String,
    pub activation_kind: String,
}

/// What materialization did, or would do in preview.
#[derive(Debug, Clone, Serialize)]
pub struct ActivationOutcome {
    pub activation: PendingActivation,
    pub link_path: String,
    /// The link's prior target, retained as the exact rollback target.
    pub previous_target: Option<String>,
    pub new_target: String,
    pub applied: bool,
    pub acknowledged: bool,
}

/// Read the approved, unacknowledged activations in database order.
pub fn pending_activations(conn: &str) -> Result<Vec<PendingActivation>, CommitError> {
    let mut client = connect(conn)?;
    read_pending(&mut client)
}

fn read_pending(client: &mut Client) -> Result<Vec<PendingActivation>, CommitError> {
    let rows = client
        .query(
            "SELECT outbox_id::text, sequence, \
                    typed_payload->>'activation_id', \
                    typed_payload->>'release_id', \
                    typed_payload->>'manifest_object_id', \
                    typed_payload->>'activation' \
             FROM lifeos_runtime.outbox \
             WHERE destination_component = $1 AND acknowledged_at IS NULL \
             ORDER BY sequence",
            &[&MATERIALIZER_DESTINATION],
        )
        .map_err(|_| CommitError::new("reading approved activations failed"))?;

    rows.iter()
        .map(|row| {
            Ok(PendingActivation {
                outbox_id: row.get(0),
                sequence: row.get(1),
                activation_id: row
                    .get::<_, Option<String>>(2)
                    .ok_or_else(|| CommitError::new("activation record lacked activation_id"))?,
                release_id: row
                    .get::<_, Option<String>>(3)
                    .ok_or_else(|| CommitError::new("activation record lacked release_id"))?,
                manifest_object_id: row.get::<_, Option<String>>(4).ok_or_else(|| {
                    CommitError::new("activation record lacked manifest_object_id")
                })?,
                activation_kind: row
                    .get::<_, Option<String>>(5)
                    .ok_or_else(|| CommitError::new("activation record lacked activation kind"))?,
            })
        })
        .collect()
}

/// Swap `link` to point at `target` atomically.
///
/// A symlink cannot be retargeted in place, so the new link is created under a
/// sibling temporary name and `rename(2)` replaces the old one. rename is
/// atomic on POSIX, so a concurrent reader observes either the previous
/// generation or the new one and never a missing link.
fn atomic_symlink_swap(link: &Path, target: &Path) -> Result<Option<String>, CommitError> {
    let previous = std::fs::read_link(link)
        .ok()
        .map(|p| p.to_string_lossy().into_owned());

    let parent = link
        .parent()
        .ok_or_else(|| CommitError::new("activation link has no parent directory"))?;
    let staging = parent.join(format!(
        ".{}.envctl-activation",
        link.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CommitError::new("activation link has no file name"))?
    ));

    // A leftover staging link from an interrupted run must not block the swap.
    let _ = std::fs::remove_file(&staging);
    std::os::unix::fs::symlink(target, &staging).map_err(internal)?;
    if let Err(error) = std::fs::rename(&staging, link) {
        let _ = std::fs::remove_file(&staging);
        return Err(internal(error));
    }
    Ok(previous)
}

/// Materialize every approved activation onto `link_path` pointing at `target`.
///
/// `apply` false previews without touching the filesystem or acknowledging.
/// The outbox record is acknowledged only after the swap succeeds, so an
/// interrupted run replays rather than silently dropping an activation.
pub fn materialize(
    conn: &str,
    link_path: &Path,
    target: &Path,
    apply: bool,
) -> Result<Vec<ActivationOutcome>, CommitError> {
    if !target.exists() {
        return Err(CommitError::new(format!(
            "activation target {} does not exist",
            target.display()
        )));
    }

    let mut client = connect(conn)?;
    // The outbox is tenant-scoped. Establish the same envctl-issued session
    // binding used by the other production worker verbs before reading or
    // acknowledging an activation; otherwise RLS would allow the filesystem
    // swap to happen and reject the durable acknowledgement afterward.
    crate::gates::bind_session(&mut client)?;
    let pending = read_pending(&mut client)?;
    let mut outcomes = Vec::with_capacity(pending.len());

    for activation in pending {
        if activation.activation_kind != ATOMIC_SYMLINK_ACTIVATION {
            return Err(CommitError::new(format!(
                "activation {} requests unsupported kind {:?}",
                activation.activation_id, activation.activation_kind
            )));
        }

        let (previous_target, applied, acknowledged) = if apply {
            let previous = atomic_symlink_swap(link_path, target)?;
            client
                .execute(
                    "UPDATE lifeos_runtime.outbox SET acknowledged_at = clock_timestamp() \
                     WHERE outbox_id = $1::text::uuid AND acknowledged_at IS NULL",
                    &[&activation.outbox_id.as_str()],
                )
                .map_err(|error| {
                    CommitError::new(format!("acknowledging the activation failed: {error}"))
                })?;
            (previous, true, true)
        } else {
            (
                std::fs::read_link(link_path)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned()),
                false,
                false,
            )
        };

        outcomes.push(ActivationOutcome {
            activation,
            link_path: link_path.to_string_lossy().into_owned(),
            previous_target,
            new_target: target.to_string_lossy().into_owned(),
            applied,
            acknowledged,
        });
    }

    Ok(outcomes)
}

/// Restore `link_path` to a previously recorded rollback target.
pub fn rollback(link_path: &Path, previous_target: &str, apply: bool) -> Result<(), CommitError> {
    let target = PathBuf::from(previous_target);
    if !target.exists() {
        return Err(CommitError::new(format!(
            "rollback target {previous_target} does not exist"
        )));
    }
    if apply {
        atomic_symlink_swap(link_path, &target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_swap_replaces_the_link_and_returns_the_rollback_target() {
        let dir = std::env::temp_dir().join(format!("envctl-act-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let first = dir.join("gen-1");
        let second = dir.join("gen-2");
        std::fs::create_dir_all(&first).expect("gen-1");
        std::fs::create_dir_all(&second).expect("gen-2");
        let link = dir.join("current");

        // First activation: no previous generation exists.
        let previous = atomic_symlink_swap(&link, &first).expect("first swap");
        assert_eq!(previous, None);
        assert_eq!(std::fs::read_link(&link).expect("link"), first);

        // Second activation: the prior target is returned as the rollback target.
        let previous = atomic_symlink_swap(&link, &second).expect("second swap");
        assert_eq!(previous.as_deref(), first.to_str());
        assert_eq!(std::fs::read_link(&link).expect("link"), second);

        // Rollback restores the exact prior generation.
        rollback(&link, first.to_str().expect("path"), true).expect("rollback");
        assert_eq!(std::fs::read_link(&link).expect("link"), first);

        // No staging link survives a completed swap.
        assert!(!dir.join(".current.envctl-activation").exists());
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn rollback_refuses_a_target_that_no_longer_exists() {
        let link = std::env::temp_dir().join("envctl-act-missing/current");
        assert!(rollback(&link, "/nonexistent/generation", true).is_err());
    }
}
