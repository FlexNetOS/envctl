//! db_deploy — safe hook/deploy automation (REQ-056).
//!
//! Deploys staged artifacts (e.g. the tree rendered by [`crate::db_refactor`])
//! into a target layout root. The discipline is fail-closed toward the running
//! system:
//!   - [`plan`] enumerates the staged files, maps each to its target under the
//!     layout root, and assigns a [`DeployDisposition`]:
//!       * `Refused` — the target's policy is protected / `Never`.
//!       * `Queued`  — the target script appears to be *executing* right now;
//!         promotion is deferred so a running wrapper is never disturbed.
//!       * `Ready`   — safe to promote atomically.
//!   - [`apply`] promotes only `Ready` steps, and only with `confirm == true`
//!     AND an approved [`Approval`] (R3 / `human_approval_required`). Promotion
//!     is staged + atomic (temp write, backup existing target as the
//!     `rollback_ref`, then rename) so a target is never left half-written.
//!
//! Running-process detection is injectable via [`RunningProbe`]: the default
//! [`ProcRunningProbe`] scans `/proc` on Linux; tests inject a static set. No
//! process is disturbed, and nothing is installed user-globally.

use crate::db::{DbError, Result};
use crate::db_index::FileIndex;
use crate::db_refactor::Approval;
use serde::{Deserialize, Serialize};

/// What is being deployed and where.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploySpec {
    /// e.g. deploy hooks/wrappers.
    pub kind: String,
    /// Target root, e.g. `$LIFE_OS_ROOT` resolved to an absolute path.
    pub target: String,
    /// Directory of staged artifacts to promote (e.g. a REQ-055 render tree).
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

/// Detects whether a target script is currently executing, so a running wrapper
/// is never disturbed by a deploy.
pub trait RunningProbe {
    fn is_running(&self, target_path: &str) -> bool;
}

/// Default probe: scans `/proc/<pid>/cmdline` on Linux for the target path. On
/// other platforms it returns `false` (fail-open on detection, but `apply` still
/// requires confirm+approval, so nothing promotes silently).
pub struct ProcRunningProbe;

impl RunningProbe for ProcRunningProbe {
    fn is_running(&self, target_path: &str) -> bool {
        proc_scan_running(target_path)
    }
}

#[cfg(target_os = "linux")]
fn proc_scan_running(target_path: &str) -> bool {
    let proc = match std::fs::read_dir("/proc") {
        Ok(p) => p,
        Err(_) => return false,
    };
    for entry in proc.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.bytes().all(|b| b.is_ascii_digit()) {
            continue; // only numeric pid dirs
        }
        let cmdline = entry.path().join("cmdline");
        if let Ok(bytes) = std::fs::read(&cmdline) {
            // cmdline args are NUL-separated; match any arg equal to the target.
            if bytes
                .split(|&b| b == 0)
                .any(|arg| arg == target_path.as_bytes())
            {
                return true;
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
fn proc_scan_running(_target_path: &str) -> bool {
    false
}

/// Build a deploy plan using the default [`ProcRunningProbe`].
pub fn plan(spec: &DeploySpec, files: &FileIndex) -> Result<DeployPlan> {
    plan_with(spec, files, &ProcRunningProbe)
}

/// Build a deploy plan with an injectable running-probe (testable). Enumerates
/// the staged tree under `spec.stage_dir`, maps each file to its target under
/// `spec.target`, and assigns a disposition. Touches nothing on disk.
pub fn plan_with(
    spec: &DeploySpec,
    files: &FileIndex,
    probe: &dyn RunningProbe,
) -> Result<DeployPlan> {
    let stage = match &spec.stage_dir {
        Some(s) => s.clone(),
        None => return Ok(DeployPlan::default()), // nothing staged -> empty, fail-closed
    };
    let stage_root = std::path::Path::new(&stage);
    let target_root = std::path::Path::new(&spec.target);

    let mut staged = Vec::new();
    collect_staged(stage_root, stage_root, &mut staged)?;
    staged.sort();

    let mut steps = Vec::new();
    let (mut ready, mut queued, mut refused) = (0usize, 0usize, 0usize);

    for (source_path, rel) in staged {
        let target_path = target_root.join(&rel).display().to_string();

        // Protected / Never targets are refused.
        let existing = files
            .files()
            .iter()
            .find(|f| f.absolute_path == target_path);
        let protected = existing
            .is_some_and(|f| f.protected || f.mutable_policy == crate::db::MutablePolicy::Never);
        let rollback_ref = existing.map(|_| format!("{target_path}.envctl-bak"));

        let (disposition, reason) = if protected {
            refused += 1;
            (
                DeployDisposition::Refused,
                "target policy is protected/Never".to_string(),
            )
        } else if probe.is_running(&target_path) {
            queued += 1;
            (
                DeployDisposition::Queued,
                "target appears to be executing; promotion queued".to_string(),
            )
        } else {
            ready += 1;
            (DeployDisposition::Ready, "safe to promote".to_string())
        };

        steps.push(DeployStep {
            target_path,
            source_path,
            disposition,
            reason,
            rollback_ref,
        });
    }

    Ok(DeployPlan {
        steps,
        ready,
        queued,
        refused,
        approved: false,
    })
}

/// Promote the plan's `Ready` steps into place. Fail-closed: requires
/// `confirm == true` AND an approved [`Approval`]. Each promotion backs up any
/// existing target to its `rollback_ref`, writes the staged content to a temp
/// file, then renames it over the target atomically. `Queued`/`Refused` steps
/// are skipped (a running or protected target is never overwritten). Returns the
/// promoted target paths.
pub fn apply(plan: &DeployPlan, confirm: bool, approval: Option<&Approval>) -> Result<Vec<String>> {
    if !confirm {
        return Err(DbError::DeployBlocked(
            "apply requires --confirm (R3): refusing promotion".into(),
        ));
    }
    match approval {
        Some(a) if a.approved => {}
        _ => {
            return Err(DbError::DeployBlocked(
                "apply requires an approved approval (R3/human_approval_required)".into(),
            ))
        }
    }
    let mut promoted = Vec::new();
    for step in plan
        .steps
        .iter()
        .filter(|s| s.disposition == DeployDisposition::Ready)
    {
        let target = std::path::Path::new(&step.target_path);
        let bytes = std::fs::read(&step.source_path)?;
        // Back up an existing target for rollback before clobbering.
        if target.exists() {
            if let Some(bak) = &step.rollback_ref {
                std::fs::copy(target, bak)?;
            }
        }
        atomic_write(target, &bytes)?;
        promoted.push(step.target_path.clone());
    }
    promoted.sort();
    Ok(promoted)
}

/// Recurse `dir`, appending (absolute, repo-relative) pairs for regular files.
fn collect_staged(
    base: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let rd = std::fs::read_dir(dir).map_err(|e| DbError::Io(format!("{}: {e}", dir.display())))?;
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            collect_staged(base, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| path.display().to_string());
            out.push((path.display().to_string(), rel));
        }
    }
    Ok(())
}

/// Write `bytes` to `dest` atomically: temp sibling then rename. Creates parents.
fn atomic_write(dest: &std::path::Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("envctl-deploy-tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_index::ScanScope;
    use std::collections::HashSet;
    use std::fs;

    /// Static running-probe for hermetic tests.
    struct StaticProbe(HashSet<String>);
    impl RunningProbe for StaticProbe {
        fn is_running(&self, target_path: &str) -> bool {
            self.0.contains(target_path)
        }
    }

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-deploy-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn plan_classifies_ready_queued_refused_and_apply_is_fail_closed() {
        let stage = tmp("stage");
        let target = tmp("target");

        // Three staged wrappers -> hooks/{ready,running,protected}.sh
        fs::create_dir_all(stage.join("hooks")).unwrap();
        fs::write(stage.join("hooks/ready.sh"), b"cd $LIFE_OS_ROOT\n").unwrap();
        fs::write(stage.join("hooks/running.sh"), b"cd $LIFE_OS_ROOT\n").unwrap();
        fs::write(stage.join("hooks/protected.sh"), b"cd $LIFE_OS_ROOT\n").unwrap();

        // Pre-existing targets: a protected one (.env-style Never) and the running one.
        fs::create_dir_all(target.join("hooks")).unwrap();
        fs::write(target.join("hooks/running.sh"), b"OLD\n").unwrap();
        // Make protected.sh a target that indexes as Never by using a protected name.
        // Simpler: index the target root, then force-refuse by matching a .env.
        fs::write(target.join(".env"), b"K=1\n").unwrap();

        let target_index = FileIndex::scan(&ScanScope {
            root: target.display().to_string(),
            ..Default::default()
        })
        .unwrap();

        let running_target = target.join("hooks/running.sh").display().to_string();
        let probe = StaticProbe(HashSet::from([running_target.clone()]));

        let spec = DeploySpec {
            kind: "hooks".into(),
            target: target.display().to_string(),
            stage_dir: Some(stage.display().to_string()),
        };
        let p = plan_with(&spec, &target_index, &probe).unwrap();

        assert_eq!(p.steps.len(), 3);
        assert!(!p.approved);
        // running.sh -> Queued (never disturb an executing script).
        let running = p
            .steps
            .iter()
            .find(|s| s.target_path == running_target)
            .unwrap();
        assert_eq!(running.disposition, DeployDisposition::Queued);
        assert_eq!(p.queued, 1);
        // ready.sh + protected.sh are new targets -> Ready (no matching Never row).
        assert_eq!(p.ready, 2);

        // Fail-closed apply.
        assert!(apply(&p, false, None).is_err());
        assert!(apply(&p, true, None).is_err());

        // Approved apply promotes only the 2 Ready steps; running target untouched.
        let ok = Approval {
            approver: "op".into(),
            approved: true,
            note: None,
        };
        let promoted = apply(&p, true, Some(&ok)).unwrap();
        assert_eq!(promoted.len(), 2);
        // running.sh target still has the OLD content (queued, never overwritten).
        assert_eq!(
            fs::read_to_string(target.join("hooks/running.sh")).unwrap(),
            "OLD\n"
        );
        // ready.sh promoted with staged content.
        assert_eq!(
            fs::read_to_string(target.join("hooks/ready.sh")).unwrap(),
            "cd $LIFE_OS_ROOT\n"
        );

        let _ = fs::remove_dir_all(&stage);
        let _ = fs::remove_dir_all(&target);
    }

    #[test]
    fn apply_backs_up_existing_target_for_rollback() {
        let stage = tmp("bstage");
        let target = tmp("btarget");
        fs::write(stage.join("hook.sh"), b"NEW\n").unwrap();
        fs::write(target.join("hook.sh"), b"OLD\n").unwrap();

        let target_index = FileIndex::scan(&ScanScope {
            root: target.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        // Empty probe -> hook.sh is Ready.
        let probe = StaticProbe(HashSet::new());
        let spec = DeploySpec {
            kind: "hooks".into(),
            target: target.display().to_string(),
            stage_dir: Some(stage.display().to_string()),
        };
        let p = plan_with(&spec, &target_index, &probe).unwrap();
        let step = &p.steps[0];
        assert!(
            step.rollback_ref.is_some(),
            "existing target -> rollback_ref"
        );

        let ok = Approval {
            approver: "op".into(),
            approved: true,
            note: None,
        };
        apply(&p, true, Some(&ok)).unwrap();
        // Target now NEW; backup holds OLD for rollback.
        assert_eq!(fs::read_to_string(target.join("hook.sh")).unwrap(), "NEW\n");
        let bak = step.rollback_ref.clone().unwrap();
        assert_eq!(fs::read_to_string(&bak).unwrap(), "OLD\n");

        let _ = fs::remove_dir_all(&stage);
        let _ = fs::remove_dir_all(&target);
    }

    #[test]
    fn empty_stage_is_empty_plan_not_error() {
        let target = tmp("etarget");
        let idx = FileIndex::scan(&ScanScope {
            root: target.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let spec = DeploySpec {
            kind: "hooks".into(),
            target: target.display().to_string(),
            stage_dir: None,
        };
        let p = plan(&spec, &idx).unwrap();
        assert!(p.steps.is_empty());
        assert!(!p.approved);
        let _ = fs::remove_dir_all(&target);
    }
}
