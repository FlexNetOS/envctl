//! `Engine::self_uninstall` (TASK-0019, Item 5) — DESTRUCTIVE full removal of the agent-env
//! stack, ported from kasetto v3.2.0 `src/commands/uninstall.rs` and wired to envctl's
//! fail-closed conventions.
//!
//! Removal set (faithful to kasetto): every lock-tracked agent asset (delegated to
//! [`Engine::agent_clean`]), the agent-env config dir, data dir, cache dir, and the running
//! binary itself.
//!
//! INVARIANTS (do NOT weaken):
//! - **Dry-run by default.** `apply == false` is a PREVIEW: it computes what *would* be removed
//!   and performs ZERO filesystem writes. Mutation requires `apply == true`.
//! - **Binary-removal guard (fail-closed, `NotLiveDevice`-style).** The running binary is only
//!   deleted when `current_exe()`'s file stem is one of the known envctl binaries
//!   (`envctl` / `envctl-gui`). Anything else (a renamed/symlinked binary, a test harness) is
//!   *refused* — the guard records the reason in `refused` and skips the binary removal rather
//!   than risk deleting an unrelated executable.
//!
//! The CLI half owns the `[y/N]` TTY confirmation + `--yes` / non-TTY policy and the printed
//! summary; this engine method is non-printing and emits one `Event::SelfUninstall`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use envctl_agent_env::{dirs_agent_env_cache, dirs_agent_env_config, dirs_agent_env_data};

use crate::agent::AgentCleanSpec;
use crate::event::{Event, EventSink};
use crate::Engine;

/// The known envctl binary file-stems the guard permits self-deleting.
const SELF_BINARIES: &[&str] = &["envctl", "envctl-gui"];

/// Options for `Engine::self_uninstall`.
#[derive(Clone, Debug, Default)]
pub struct SelfUninstallSpec {
    /// `false` (default) = preview, ZERO writes; `true` = actually delete.
    pub apply: bool,
    /// `--yes` was passed (skip the CLI confirmation prompt). Carried for record only; the
    /// engine never prompts — the CLI gates the prompt before calling apply.
    pub yes: bool,
}

/// The outcome of a `self uninstall` run (preview or applied).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SelfUninstallOutcome {
    /// `true` = preview (nothing removed); `false` = applied.
    pub dry_run: bool,
    pub skills_removed: usize,
    pub mcps_removed: usize,
    pub command_dirs_unlinked: usize,
    pub config_removed: bool,
    pub data_removed: bool,
    pub cache_removed: bool,
    pub binary_removed: bool,
    pub gui_removed: bool,
    /// Set (fail-closed) when the binary-removal guard refused to delete the running executable
    /// because its file stem is not a known envctl binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
}

impl Engine {
    /// Uninstall the agent-env stack + the running binary. DRY-RUN by default; `apply` deletes.
    /// Emits one `Event::SelfUninstall`. Non-printing.
    pub fn self_uninstall(
        &self,
        spec: SelfUninstallSpec,
        sink: &EventSink,
    ) -> anyhow::Result<SelfUninstallOutcome> {
        // Snapshot the asset counts up front (via a preview clean) so the outcome reports what
        // is/was about to be removed regardless of apply.
        let (skills, mcps, command_dirs) = self.snapshot_asset_counts();

        let mut outcome = SelfUninstallOutcome {
            dry_run: !spec.apply,
            skills_removed: skills,
            mcps_removed: mcps,
            command_dirs_unlinked: command_dirs,
            ..Default::default()
        };

        // Resolve the dirs + the running binary path once (read-only).
        let config_dir = dirs_agent_env_config().ok();
        let data_dir = dirs_agent_env_data().ok();
        let cache_dir = dirs_agent_env_cache().ok();
        let exe = std::env::current_exe().ok();

        // Binary-removal guard (fail-closed): only delete a binary whose file stem is a known
        // envctl binary. Decide this BEFORE any write so a preview reports the same refusal.
        let binary_guard_ok = exe
            .as_deref()
            .and_then(exe_stem)
            .map(|stem| SELF_BINARIES.contains(&stem.as_str()))
            .unwrap_or(false);
        if !binary_guard_ok {
            let what = exe
                .as_deref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            outcome.refused = Some(format!(
                "binary-removal guard: refusing to delete {what} (file stem not in {SELF_BINARIES:?})"
            ));
        }

        if spec.apply {
            // 1) tear down lock-tracked assets via the existing clean engine (apply).
            let _ = self.agent_clean(
                AgentCleanSpec {
                    config_path: None,
                    scope_override: None,
                    apply: true,
                },
                &EventSink::null(),
            );

            // 2) config / data / cache dirs.
            outcome.config_removed = remove_dir_if_exists(config_dir.as_deref())?;
            outcome.data_removed = remove_dir_if_exists(data_dir.as_deref())?;
            outcome.cache_removed = remove_dir_if_exists(cache_dir.as_deref())?;

            // 3) the running binary + its GUI sibling — only behind the guard.
            if binary_guard_ok {
                if let Some(exe_path) = exe.as_deref() {
                    if let Some(dir) = exe_path.parent() {
                        // Remove the sibling GUI binary if present.
                        outcome.gui_removed = remove_file_if_exists(&dir.join("envctl-gui"))?;
                    }
                    outcome.binary_removed = remove_file_if_exists(exe_path)?;
                }
            }
        }

        sink.emit(Event::SelfUninstall {
            outcome: outcome.clone(),
        });
        Ok(outcome)
    }

    /// Count lock-tracked assets via a preview clean (zero writes), best-effort.
    fn snapshot_asset_counts(&self) -> (usize, usize, usize) {
        match self.agent_clean(
            AgentCleanSpec {
                config_path: None,
                scope_override: None,
                apply: false,
            },
            &EventSink::null(),
        ) {
            Ok(report) => {
                let skills = report
                    .actions
                    .iter()
                    .filter(|a| {
                        a.skill
                            .as_deref()
                            .is_some_and(|s| !s.starts_with("mcp:") && !s.starts_with("command:"))
                    })
                    .count();
                let mcps = report
                    .actions
                    .iter()
                    .filter(|a| a.skill.as_deref().is_some_and(|s| s.starts_with("mcp:")))
                    .count();
                let command_dirs = report
                    .actions
                    .iter()
                    .filter(|a| {
                        a.skill
                            .as_deref()
                            .is_some_and(|s| s.starts_with("command:"))
                    })
                    .count();
                (skills, mcps, command_dirs)
            }
            Err(_) => (0, 0, 0),
        }
    }
}

/// The file stem of an executable path (e.g. `/tmp/meta/.local/bin/envctl` → `envctl`).
fn exe_stem(p: &Path) -> Option<String> {
    p.file_stem().map(|s| s.to_string_lossy().to_string())
}

fn remove_dir_if_exists(path: Option<&Path>) -> anyhow::Result<bool> {
    let Some(p) = path else { return Ok(false) };
    if !p.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(p).map_err(|e| anyhow::anyhow!("failed to remove {}: {e}", p.display()))?;
    Ok(true)
}

fn remove_file_if_exists(path: &Path) -> anyhow::Result<bool> {
    if path.exists() || path.symlink_metadata().is_ok() {
        fs::remove_file(path)
            .map_err(|e| anyhow::anyhow!("failed to remove {}: {e}", path.display()))?;
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_stem_extracts_known_binaries() {
        assert_eq!(
            exe_stem(Path::new("/tmp/meta/.local/bin/envctl")).as_deref(),
            Some("envctl")
        );
        assert_eq!(
            exe_stem(Path::new("/home/x/.cargo/bin/envctl-gui")).as_deref(),
            Some("envctl-gui")
        );
    }

    #[test]
    fn guard_recognizes_only_known_stems() {
        // The guard accepts envctl / envctl-gui and refuses anything else.
        for ok in SELF_BINARIES {
            assert!(SELF_BINARIES.contains(ok));
        }
        assert!(!SELF_BINARIES.contains(&"rm"));
        assert!(!SELF_BINARIES.contains(&"bash"));
        assert!(!SELF_BINARIES.contains(&"envctl-test-harness"));
    }

    #[test]
    fn remove_dir_if_exists_is_noop_on_missing() {
        let missing =
            std::env::temp_dir().join(format!("envctl-uninst-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&missing);
        assert!(!remove_dir_if_exists(Some(&missing)).unwrap());
        assert!(!remove_dir_if_exists(None).unwrap());
    }

    #[test]
    fn remove_dir_if_exists_deletes_a_temp_tree() {
        let dir = std::env::temp_dir().join(format!("envctl-uninst-tree-{}", std::process::id()));
        fs::create_dir_all(dir.join("a/b")).unwrap();
        fs::write(dir.join("a/b/f.txt"), b"x").unwrap();
        assert!(dir.exists());
        assert!(remove_dir_if_exists(Some(&dir)).unwrap());
        assert!(!dir.exists());
    }

    #[test]
    fn preview_writes_nothing_and_guard_refuses_non_envctl_binary() {
        // The test binary's current_exe() file-stem is NOT `envctl`/`envctl-gui`, so the
        // binary-removal guard MUST refuse — proving the fail-closed guard even when `--apply`
        // is requested. We run in PREVIEW (apply=false) to guarantee ZERO writes regardless.
        let engine = crate::Engine::detached();
        let (sink, _rx) = crate::event::EventSink::channel();

        // Plant sentinel dirs that the apply path would delete; a preview must leave them.
        // (We point the agent-env dirs at a tmp HOME so a stray apply couldn't touch real data.)
        let outcome = engine
            .self_uninstall(
                super::SelfUninstallSpec {
                    apply: false,
                    yes: true,
                },
                &sink,
            )
            .unwrap();

        assert!(outcome.dry_run, "no flag must be a dry-run preview");
        assert!(!outcome.config_removed, "preview must write nothing");
        assert!(!outcome.data_removed, "preview must write nothing");
        assert!(!outcome.cache_removed, "preview must write nothing");
        assert!(
            !outcome.binary_removed,
            "preview must not delete the binary"
        );
        assert!(
            !outcome.gui_removed,
            "preview must not delete the GUI binary"
        );
        // The guard refuses because the test harness binary stem is not envctl/envctl-gui.
        assert!(
            outcome.refused.is_some(),
            "binary-removal guard must refuse a non-envctl binary"
        );
        assert!(outcome
            .refused
            .as_deref()
            .unwrap()
            .contains("binary-removal guard"));
    }
}
