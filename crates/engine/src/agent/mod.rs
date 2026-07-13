//! The agent-env subsystem: the 6 `Engine` agent-asset verbs
//! (`agent_{sync,add,remove,lock,list,clean}`) that drive the pure-Rust `envctl-agent-env`
//! library. The engine stays sync + non-printing: each verb emits `Event::Agent*` and returns
//! typed data; the front-ends (TASK-0014 CLI/GUI) build the `Agent*Spec` opts identically and
//! drain the EventSink — so they can never diverge.
//!
//! Fail-closed policy lives here, not in the library:
//! - every mutating verb (`sync`/`add`/`remove`/`clean`) defaults to **preview** (`apply:
//!   false`) and performs ZERO filesystem writes until `apply: true`;
//! - `lock_mode = Locked` is **zero-network** (the driver refuses any source that would need a
//!   fetch, recording a `locked_error` and bumping `failed`);
//! - pruning is **never-on-failure** (the library's `remove_stale` only runs when
//!   `summary.failed == 0`).
//!
//! The agent-asset lock (`agent-env.lock`, SHA-256) is wholly separate from the engine's
//! FNV-1a component lock (`crate::lock`): this module never imports `crate::lock`.

pub mod clean;
pub mod doctor;
pub mod edit;
pub mod init;
pub mod list;
pub mod lock;
pub mod report;
pub mod sync;

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use envctl_agent_env::{
    config_path::default_config_path,
    extend::{load_config_any, load_config_any_zero_network},
    fsops::{resolve_destinations, scope_root},
    lock::{lock_path, LockMode},
    Config, Scope,
};

/// Re-export the per-verb spec/return types so callers `use crate::agent::*`.
pub use doctor::AgentDoctorSpec;
pub use report::{
    AgentCommandDirCheck, AgentDoctorReport, AgentEditItem, AgentEditOutcome, AgentInitOutcome,
    AgentList, AgentLockDriftItem, AgentLockOutcome, AgentReport, AgentUpdateCheck, AgentVerb,
};

/// Serializable mirror of the library `Scope` — the engine-facing scope (so `event.rs` and
/// the spec types don't leak the library type directly into the Event vocabulary).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentScope {
    Global,
    Project,
}

impl From<Scope> for AgentScope {
    fn from(s: Scope) -> Self {
        match s {
            Scope::Global => AgentScope::Global,
            Scope::Project => AgentScope::Project,
        }
    }
}

impl From<AgentScope> for Scope {
    fn from(s: AgentScope) -> Self {
        match s {
            AgentScope::Global => Scope::Global,
            AgentScope::Project => Scope::Project,
        }
    }
}

/// The lock mode an agent verb runs in (front-end maps `--locked`/`--update[names]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentLockMode {
    /// Verify + fetch as needed, write/refresh the lock (default).
    Plain,
    /// Re-resolve the named packages' refs and rewrite the lock (empty = all).
    Update { only: Vec<String> },
    /// Verify the lock is satisfied with ZERO network fetch; fail-closed if unsatisfied.
    Locked,
}

impl AgentLockMode {
    /// Map the `--locked` / `--update [names]` front-end flags to an `AgentLockMode`. Shared by
    /// the CLI and GUI so the two front-ends derive the lock mode from one source (parity).
    /// `--locked` wins (zero-network audit); `--update [names]` re-resolves the named packages
    /// (empty = all); otherwise the default `Plain` (verify+fetch, write lock).
    pub fn from_flags(locked: bool, update: Option<Vec<String>>) -> AgentLockMode {
        if locked {
            AgentLockMode::Locked
        } else if let Some(only) = update {
            AgentLockMode::Update { only }
        } else {
            AgentLockMode::Plain
        }
    }

    pub(crate) fn to_library(&self) -> LockMode {
        match self {
            AgentLockMode::Plain => LockMode::Plain,
            AgentLockMode::Update { only } => LockMode::Update(only.clone()),
            AgentLockMode::Locked => LockMode::Locked,
        }
    }

    pub(crate) fn label(&self) -> String {
        match self {
            AgentLockMode::Plain => "plain".into(),
            AgentLockMode::Update { only } if only.is_empty() => "update".into(),
            AgentLockMode::Update { only } => format!("update:{}", only.join(",")),
            AgentLockMode::Locked => "locked".into(),
        }
    }
}

/// Which asset kinds a `list` shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentListKind {
    All,
    Skills,
    Mcps,
    Commands,
}

// --------------------------------------------------------------------------------------
// Spec types (the typed opts the front-ends build from clap/GUI; parity contract)
// --------------------------------------------------------------------------------------

/// Options for `Engine::agent_sync`.
#[derive(Clone, Debug)]
pub struct AgentSyncSpec {
    /// Config file path; `None` → the M-22 default-config resolution.
    pub config_path: Option<String>,
    /// `--scope` override; `None` → resolved from the config.
    pub scope_override: Option<AgentScope>,
    /// `false` (the default) = preview/dry-run, ZERO writes; `true` = apply.
    pub apply: bool,
    pub lock_mode: AgentLockMode,
}

impl Default for AgentSyncSpec {
    fn default() -> Self {
        AgentSyncSpec {
            config_path: None,
            scope_override: None,
            apply: false,
            lock_mode: AgentLockMode::Plain,
        }
    }
}

/// Per-kind named selectors for `add`/`remove` (the `--skill`/`--mcp`/`--command` flags).
#[derive(Clone, Debug, Default)]
pub struct AgentSectionSel {
    pub skills: Vec<String>,
    pub mcps: Vec<String>,
    pub commands: Vec<String>,
}

/// Options for `Engine::agent_add`.
#[derive(Clone, Debug)]
pub struct AgentAddSpec {
    pub source: String,
    pub section: AgentSectionSel,
    pub git_ref: Option<String>,
    pub branch: Option<String>,
    pub sub_dir: Option<String>,
    pub config_path: Option<String>,
    pub scope_override: Option<AgentScope>,
    pub apply: bool,
    pub no_sync: bool,
    pub no_verify: bool,
    pub lock_mode: AgentLockMode,
}

/// Options for `Engine::agent_remove`.
#[derive(Clone, Debug)]
pub struct AgentRemoveSpec {
    pub source: String,
    pub section: AgentSectionSel,
    pub git_ref: Option<String>,
    pub branch: Option<String>,
    pub sub_dir: Option<String>,
    pub config_path: Option<String>,
    pub scope_override: Option<AgentScope>,
    pub apply: bool,
    pub no_sync: bool,
    pub lock_mode: AgentLockMode,
}

/// Options for `Engine::agent_lock`.
#[derive(Clone, Debug)]
pub struct AgentLockSpec {
    pub config_path: Option<String>,
    pub scope_override: Option<AgentScope>,
    /// `--check`: verify the lock matches the config without writing.
    pub check: bool,
    /// `--upgrade-package <name>...`: restrict re-resolve to sources providing these skills.
    pub upgrade_only: Vec<String>,
    /// Honored only with `--check`: `Locked` makes the audit zero-network.
    pub lock_mode: AgentLockMode,
}

/// Options for `Engine::agent_list`.
#[derive(Clone, Debug)]
pub struct AgentListSpec {
    pub scope_override: Option<AgentScope>,
    pub kind: AgentListKind,
}

/// Options for `Engine::agent_clean`.
#[derive(Clone, Debug)]
pub struct AgentCleanSpec {
    pub config_path: Option<String>,
    pub scope_override: Option<AgentScope>,
    pub apply: bool,
}

/// Options for `Engine::agent_init`.
#[derive(Clone, Debug)]
pub struct AgentInitSpec {
    /// Write the global config under `$XDG_CONFIG_HOME/agent-env/agent-env.yaml`
    /// instead of `./agent-env.yaml`.
    pub global: bool,
    /// Overwrite an existing config file (default: fail closed).
    pub force: bool,
}

// --------------------------------------------------------------------------------------
// Shared resolution context (the engine analog of kasetto's SyncContext bookkeeping)
// --------------------------------------------------------------------------------------

/// The resolved config + scope + destinations for a verb run, plus the agent-asset lock path.
/// Built by [`AgentCtx::resolve`] from a config path (with the M-22 file-read fallback) and an
/// optional scope override.
pub(crate) struct AgentCtx {
    pub cfg: Config,
    pub cfg_dir: PathBuf,
    pub cfg_label: String,
    pub scope: Scope,
    pub destinations: Vec<PathBuf>,
    pub scope_root: PathBuf,
    pub lock_file: PathBuf,
}

impl AgentCtx {
    /// Resolve the run context from a config path + scope override.
    ///
    /// **M-22 fallback:** when `config_path` is `None`, the default config path is computed
    /// (`config_path::default_config_path`) and loaded via `load_config_any` (which walks
    /// `extends`); the scope then resolves from the loaded config (`Config::resolved_scope`)
    /// unless overridden. This is the single entry seam both the CLI and GUI share.
    pub(crate) fn resolve(
        config_path: Option<&str>,
        scope_override: Option<AgentScope>,
    ) -> anyhow::Result<AgentCtx> {
        Self::resolve_with_policy(config_path, scope_override, true)
    }

    /// Resolve a context while refusing remote root configs and remote `extends` before
    /// any HTTP request is constructed. Used by `agent lock --check --locked`.
    pub(crate) fn resolve_zero_network(
        config_path: Option<&str>,
        scope_override: Option<AgentScope>,
    ) -> anyhow::Result<AgentCtx> {
        Self::resolve_with_policy(config_path, scope_override, false)
    }

    fn resolve_with_policy(
        config_path: Option<&str>,
        scope_override: Option<AgentScope>,
        allow_network: bool,
    ) -> anyhow::Result<AgentCtx> {
        let path = config_path
            .map(str::to_string)
            .unwrap_or_else(default_config_path);
        let loaded = if allow_network {
            load_config_any(&path)
        } else {
            load_config_any_zero_network(&path)
        };
        let (cfg, cfg_dir, cfg_label) =
            loaded.with_context(|| format!("failed to load agent config from {path}"))?;
        let scope = match scope_override {
            Some(s) => s.into(),
            None => cfg.resolved_scope(),
        };
        let destinations = resolve_destinations(&cfg_dir, &cfg, scope)
            .with_context(|| "failed to resolve agent destinations")?;
        let scope_root =
            scope_root(scope, &cfg_dir).with_context(|| "failed to resolve agent scope root")?;
        let lock_file = agent_lock_path(scope, &cfg_dir)
            .with_context(|| "failed to resolve agent lock path")?;
        Ok(AgentCtx {
            cfg,
            cfg_dir,
            cfg_label,
            scope,
            destinations,
            scope_root,
            lock_file,
        })
    }
}

/// Resolve the standalone `agent-env.lock` path for a scope, anchored on the project root
/// (`cfg_dir`) and the global agent-env data dir. Separate from the FNV-1a component lock.
pub(crate) fn agent_lock_path(scope: Scope, cfg_dir: &Path) -> anyhow::Result<PathBuf> {
    let global = envctl_agent_env::dirs::dirs_agent_env_data()?;
    Ok(lock_path(scope, cfg_dir, &global))
}

#[cfg(test)]
mod tests {
    use super::AgentLockMode;

    // Moved from cli/src/main.rs (TASK-0014b open-Q1): the lock-mode flag mapping now lives in
    // the engine so the CLI and GUI share one source. Both front-ends call
    // `AgentLockMode::from_flags`.
    #[test]
    fn from_flags_maps_each_flag() {
        // default: no --locked, no --update -> Plain
        assert_eq!(AgentLockMode::from_flags(false, None), AgentLockMode::Plain);
        // --locked wins (zero-network audit)
        assert_eq!(AgentLockMode::from_flags(true, None), AgentLockMode::Locked);
        // --locked beats --update if both somehow set
        assert_eq!(
            AgentLockMode::from_flags(true, Some(vec![])),
            AgentLockMode::Locked
        );
        // --update with no names -> Update { only: [] } (= all)
        assert_eq!(
            AgentLockMode::from_flags(false, Some(vec![])),
            AgentLockMode::Update { only: vec![] }
        );
        // --update foo bar -> Update { only: [foo, bar] }
        assert_eq!(
            AgentLockMode::from_flags(false, Some(vec!["foo".into(), "bar".into()])),
            AgentLockMode::Update {
                only: vec!["foo".into(), "bar".into()]
            }
        );
    }
}
