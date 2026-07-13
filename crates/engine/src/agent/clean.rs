//! `Engine::agent_clean` (C-11/C-14) — tear down every lock-tracked agent asset (skill dirs,
//! command files, and the MCP servers THIS lock installed) and reset the lock + runtime.
//!
//! Preview (`apply: false`) reports the counts/would-remove actions and writes NOTHING. Apply
//! (`apply: true`) deletes the tracked assets, clears + saves the lock, and clears the runtime.
//!
//! MCP teardown only removes **lock-tracked** servers (`remove_mcp_server`), so pre-existing
//! global MCP servers (broker/repowire/weave, never in the agent lock) are never touched
//! (C-14 never-clobber). Binary-self-removal is out of scope (env-manager binary stays).

use envctl_agent_env::driver::{clean_actions, clean_apply_transaction, UpdatedAt};
use envctl_agent_env::lock::AgentLockFile;
use envctl_agent_env::report::Summary;

use crate::agent::report::{AgentReport, AgentVerb};
use crate::agent::{AgentCleanSpec, AgentCtx};
use crate::event::{Event, EventSink};
use crate::Engine;

impl Engine {
    /// Clean every lock-tracked agent asset and reset the lock. DRY-RUN by default.
    ///
    /// The run is anchored on the resolved config directory (the directory that holds the
    /// loaded `agent-env.yaml`), so `agent clean` works from any subdirectory of a project —
    /// it finds the lock relative to the config root, not relative to `std::env::current_dir()`.
    pub fn agent_clean(
        &self,
        spec: AgentCleanSpec,
        sink: &EventSink,
    ) -> anyhow::Result<AgentReport> {
        let ctx = AgentCtx::resolve(spec.config_path.as_deref(), spec.scope_override)?;
        let scope = ctx.scope;
        let project_root = &ctx.cfg_dir;
        let lock_file = &ctx.lock_file;
        let mut lock: AgentLockFile = envctl_agent_env::lock::load(lock_file)?;

        sink.emit(Event::AgentRunStarted {
            verb: AgentVerb::Clean,
            scope: scope.into(),
            dry_run: !spec.apply,
            lock_mode: "plain".into(),
        });

        let runtime = envctl_agent_env::runtime::load_runtime_state(scope, project_root)?;
        let mut updated = UpdatedAt {
            installed_at: runtime.installed_at,
            managed_outputs: runtime.managed_outputs,
            last_run: runtime.last_run,
            latest_report: runtime.latest_report,
        };
        let (counts, mut actions) =
            clean_actions(&lock, &updated, scope, project_root, &ctx.destinations)?;
        let status = if spec.apply {
            "removed"
        } else {
            "would_remove"
        };
        for action in &mut actions {
            action.status = status.into();
        }

        if spec.apply {
            clean_apply_transaction(
                &mut lock,
                &mut updated,
                scope,
                project_root,
                &ctx.destinations,
            )?;
        }

        let summary = Summary {
            removed: counts.skills_removed + counts.mcps_removed + counts.commands_removed,
            ..Default::default()
        };

        for a in &actions {
            sink.emit(Event::AgentAction {
                source: a.source.clone(),
                asset: a.skill.clone(),
                status: a.status.clone(),
                error: a.error.clone(),
            });
        }

        let report = AgentReport {
            run_id: envctl_agent_env::util::now_unix().to_string(),
            config: lock_file.to_string_lossy().to_string(),
            destination: project_root.to_string_lossy().to_string(),
            scope: scope.into(),
            dry_run: !spec.apply,
            summary,
            actions,
        };
        sink.emit(Event::AgentRunFinished {
            report: report.clone(),
        });
        Ok(report)
    }
}
