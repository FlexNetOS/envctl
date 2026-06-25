//! `meta-env` — envctl's meta subprocess plugin.
//!
//! Native adoption of `meta_plugin_protocol`: this makes envctl a first-class meta
//! plugin so `meta env <verb>` dispatches into envctl (alongside `meta git`,
//! `meta project`, `meta rust`, `meta dashboard`). The plugin is owned and shipped
//! BY envctl (this bin lives in the `envctl` CLI crate), not by a separate meta crate.
//!
//! Mechanism (the canonical meta plugin pattern, same as `meta-git`): `execute`
//! returns an `ExecutionPlan` whose single command runs the `envctl` binary for the
//! requested verb. The host runs it via loop_lib, so envctl's own CLI rendering and
//! its fail-closed / dry-run-by-default destructive semantics are reused verbatim —
//! the plugin never re-implements them and the two surfaces cannot diverge.
//!
//! Logging is initialized by `run_plugin()`. Use `RUST_LOG=meta_env=debug`.

use indexmap::IndexMap;
use meta_plugin_protocol::{
    run_plugin, CommandResult, PlannedCommand, PluginDefinition, PluginHelp, PluginInfo,
    PluginRequest,
};

/// envctl's verbs, exposed under the `env` namespace. (Read-only: `auto-detect`,
/// `graph`, `lock`, `doctor`; mutating verbs are dry-run by default and require the
/// user to pass `--apply`/`--build`, which flows through verbatim in `args`.)
const VERBS: &[(&str, &str)] = &[
    (
        "auto-detect",
        "Read-only environment inventory (host/GPU/tools/drift)",
    ),
    (
        "install",
        "Bring components to present+verified (idempotent)",
    ),
    (
        "auto-fix",
        "Repair broken/partial components (dry-run; --apply to act)",
    ),
    (
        "reset",
        "Uninstall/unwire toward baseline (dry-run; --apply to act)",
    ),
    (
        "add-repo",
        "Build a repo from source + wire it in (preview; --build to act)",
    ),
    ("graph", "Component dependency-DAG intelligence"),
    ("lock", "Content-hashed envctl.lock (+ --check CI gate)"),
    (
        "doctor",
        "Read-only health: writability, toolchains, GPU, last-op",
    ),
    ("agent", "agent-env provisioning verbs (sync/lock/...)"),
];

fn main() {
    let mut commands: IndexMap<String, String> = IndexMap::new();
    for (verb, desc) in VERBS {
        commands.insert(format!("env {verb}"), desc.to_string());
    }

    run_plugin(PluginDefinition {
        info: PluginInfo {
            name: "env".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: VERBS.iter().map(|(v, _)| format!("env {v}")).collect(),
            description: Some(
                "Environment management for the meta workspace (envctl)".to_string(),
            ),
            help: Some(PluginHelp {
                usage: "meta env - envctl, the meta environment manager\n\nUsage: meta env <VERB> [OPTIONS]".to_string(),
                commands,
                command_sections: IndexMap::new(),
                examples: vec![
                    "meta env doctor".to_string(),
                    "meta env auto-detect --json".to_string(),
                    "meta env install".to_string(),
                    "meta env auto-fix --apply".to_string(),
                ],
                note: Some(
                    "Dispatches to the `envctl` binary; destructive verbs stay dry-run by default (pass --apply/--build).".to_string(),
                ),
            }),
        },
        execute,
    });
}

fn execute(request: PluginRequest) -> CommandResult {
    // The host sends the matched command, namespace-prefixed (e.g. "env doctor").
    // Strip the leading `env` token to recover the envctl verb (+ any subcommand).
    let verb = request
        .command
        .trim()
        .strip_prefix("env")
        .map(str::trim)
        .unwrap_or_else(|| request.command.trim());
    if verb.is_empty() {
        return CommandResult::ShowHelp(None);
    }

    // Build `envctl <verb> <args> [--json]`. Args (incl. any --apply/--build the user
    // typed) flow through verbatim, preserving envctl's fail-closed mutation gating.
    let mut parts = vec!["envctl".to_string(), verb.to_string()];
    parts.extend(request.args.iter().cloned());
    if request.options.json_output {
        parts.push("--json".to_string());
    }
    let cmd = parts.join(" ");

    // envctl manages the box once (it is not a per-repo fan-out); run in the host's cwd.
    let dir = if request.cwd.is_empty() {
        ".".to_string()
    } else {
        request.cwd.clone()
    };

    CommandResult::Plan(
        vec![PlannedCommand {
            dir,
            cmd,
            env: None,
        }],
        Some(false), // never parallelize a single box-management command
    )
}
