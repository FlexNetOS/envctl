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

    // envctl manages the box once (it is not a per-repo fan-out). It must run with its OWN repo as
    // the working dir so its default `./manifest` resolves — the host may dispatch us from the meta
    // root or any child repo, where `./manifest` does not exist. Resolve the meta root from the host
    // cwd via the `.meta.yaml` marker (like git finds `.git`) and run in `<meta_root>/envctl`. Fall
    // back to the host cwd when no marker is found, so nothing regresses outside a meta tree.
    let start = if request.cwd.is_empty() {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    } else {
        std::path::PathBuf::from(&request.cwd)
    };
    let dir = match find_meta_root(&start) {
        Some(root) => root.join("envctl").to_string_lossy().into_owned(),
        None if request.cwd.is_empty() => ".".to_string(),
        None => request.cwd.clone(),
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

/// Walk up from `start` to the first ancestor containing a `.meta.yaml` marker (the meta workspace
/// root), like git resolving `.git`. Returns `None` outside a meta tree. Kept dependency-free (the
/// plugin bin only links `meta_plugin_protocol`); the engine has its own richer resolver.
fn find_meta_root(start: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(start)
    };
    loop {
        if dir.join(".meta.yaml").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_meta_root;
    use std::fs;

    #[test]
    fn find_meta_root_walks_up_to_marker_and_none_without_one() {
        // <base>/root/.meta.yaml  +  <base>/root/envctl/crates/cli  (a child dir to walk up from)
        let base =
            std::env::temp_dir().join(format!("envctl-meta-env-test-{}", std::process::id()));
        let root = base.join("root");
        let child = root.join("envctl/crates/cli");
        fs::create_dir_all(&child).unwrap();
        fs::write(root.join(".meta.yaml"), "projects: {}\n").unwrap();

        // From deep inside the tree, the resolver finds the marker-bearing root.
        assert_eq!(find_meta_root(&child).as_deref(), Some(root.as_path()));
        // The root itself also resolves to itself.
        assert_eq!(find_meta_root(&root).as_deref(), Some(root.as_path()));
        // A sibling tree with no marker resolves to None (no ancestor has `.meta.yaml`).
        let bare = base.join("bare/sub");
        fs::create_dir_all(&bare).unwrap();
        assert_eq!(find_meta_root(&bare), None);

        let _ = fs::remove_dir_all(&base);
    }
}
