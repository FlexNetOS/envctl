#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn envctl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("harness crate path has envctl ancestor")
        .to_path_buf()
}

fn parse_toml(path: &Path) -> toml::Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    toml::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn string_at<'a>(value: &'a toml::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(toml::Value::as_str)
}

fn tree_has_files(path: &Path) -> bool {
    if path.is_file() {
        return true;
    }
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|entry| tree_has_files(&entry.path()))
        })
        .unwrap_or(false)
}

#[test]
fn tracked_policy_is_a_complete_clean_checkout_grant() {
    let root = envctl_root();
    let policy = parse_toml(&root.join("home/agent-env/codex-harness/policy/policy.toml"));
    let grants = policy
        .get("permission_grants")
        .expect("permission_grants table");

    assert_eq!(
        string_at(&policy, "full_access_decision_id"),
        Some(codex_harness::USER_FULL_ACCESS_DECISION_ID)
    );
    assert_eq!(
        string_at(grants, "decision_id"),
        Some(codex_harness::USER_FULL_ACCESS_DECISION_ID)
    );
    assert_eq!(string_at(grants, "danger_full_access"), Some("keep"));
    assert_eq!(
        grants
            .get("operator_grants_are_execution_context")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        grants
            .get("expanded_access_is_not_a_blocker")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let library = fs::read_to_string(root.join("home/agent-env/codex-harness/src/lib.rs")).unwrap();
    assert!(
        !library.contains("|| full_access_marker_path().exists()"),
        "ignored marker must not be permission authority"
    );
}

#[test]
fn envctl_owns_no_routeable_agent_runtime_profiles() {
    let root = envctl_root();
    let retired_agent_roots = ["codex", "claude"]
        .map(|agent| format!("home/.{agent}"))
        .into_iter()
        .chain(["profile-runtime".to_string()]);
    for relative in retired_agent_roots {
        assert!(
            !tree_has_files(&root.join(&relative)),
            "envctl must not project an installed agent runtime: {relative}"
        );
    }

    let manifest = fs::read_to_string(root.join("manifest/ai-clis.toml")).unwrap();
    assert!(manifest.contains("envctl-codex-profile-lifecycle.sh"));
    assert!(manifest.contains("envctl-claude-cleanup.sh"));
    assert!(manifest.contains("envctl-profile-command-lifecycle.sh"));
}

#[test]
fn every_agent_route_preserves_full_access_context() {
    let root = envctl_root();
    let harness_agents = root.join("home/agent-env/codex-harness/agents");
    for entry in fs::read_dir(&harness_agents).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        let agent = parse_toml(&path);
        assert_eq!(
            string_at(&agent, "default_profile"),
            Some("envctl-harness"),
            "{}",
            path.display()
        );
    }

    assert!(!root.join("profile-runtime/codex/agents").exists());
}

#[test]
fn native_rule_mirrors_allow_the_operator_frontdoor() {
    let root = envctl_root();
    let relative = "home/agent-env/codex-harness/rules/no-yolo.rules";
    let text = fs::read_to_string(root.join(relative)).unwrap();
    assert!(
        text.contains("pattern = [\"codex\", \"--dangerously-bypass-approvals-and-sandbox\"]"),
        "{relative}"
    );
    assert!(text.contains("decision = \"allow\""), "{relative}");
    assert!(
        text.contains(codex_harness::USER_FULL_ACCESS_DECISION_ID),
        "{relative}"
    );
    assert!(!root.join("profile-runtime/codex/rules").exists());
}

#[test]
fn repository_agent_models_are_sol_terra_luna_or_explicit_fallbacks() {
    let root = envctl_root();
    for directory in [
        root.join("home/agent-env/codex-harness/agents"),
        root.join(".codex/agents"),
    ] {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|value| value.to_str()) != Some("toml") {
                continue;
            }
            let agent = parse_toml(&path);
            assert_ne!(
                string_at(&agent, "model"),
                Some("gpt-5.5"),
                "{} retains a routeable GPT-5.5 assignment",
                path.display()
            );
        }
    }
    for (name, expected) in [
        ("plan-architect.toml", "gpt-5.6-sol"),
        ("plan-analyst.toml", "gpt-5.6-terra"),
        ("plan-autoresearch-loop-auditor.toml", "gpt-5.6-luna"),
    ] {
        let agent = parse_toml(&root.join(".codex/agents").join(name));
        assert_eq!(string_at(&agent, "model"), Some(expected), "{name}");
    }
}

#[test]
fn retired_roots_and_tracked_model_cache_are_absent() {
    let root = envctl_root();
    assert!(!root.join("profile-runtime").exists());
    for agent in ["codex", "claude"] {
        assert!(!tree_has_files(
            &root.join("home").join(format!(".{agent}"))
        ));
    }

    let library = fs::read_to_string(root.join("home/agent-env/codex-harness/src/lib.rs")).unwrap();
    assert!(!library.contains("/home/flexnetos/lifeos/src/envctl"));
    assert!(library.contains("/home/flexnetos/meta/src/envctl/home"));

    let baseline =
        fs::read_to_string(root.join("manifest/components.d/codex-global-baseline.toml")).unwrap();
    assert!(
        baseline.contains("envctl-codex-global-baseline-lifecycle.sh"),
        "Codex policy must dispatch the read-only profile compatibility lifecycle"
    );
    assert!(!baseline.contains("LIFEOS_ROOT"));
    for forbidden in [
        "model-catalog",
        "profile-runtime/codex/agents",
        "marketplaces.",
        "plugins.\"",
        "CODEX_HOME",
        "CODEX_SQLITE_HOME",
    ] {
        assert!(
            !baseline.contains(forbidden),
            "Codex global policy regenerated forbidden runtime surface: {forbidden}"
        );
    }

    let access = parse_toml(
        &root.join("home/agent-env/codex-harness/model-catalog/model-access-matrix.toml"),
    );
    let models = access
        .get("models")
        .and_then(toml::Value::as_table)
        .expect("model access table");
    for model in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        assert_eq!(
            models
                .get(model)
                .and_then(toml::Value::as_table)
                .and_then(|entry| entry.get("account_access"))
                .and_then(toml::Value::as_str),
            Some("proved"),
            "{model} must reflect the live 2026-07-11 proof"
        );
    }
}

#[test]
fn one_skill_owns_session_controls_and_github_execution_policy() {
    let root = envctl_root();
    assert!(
        !root
            .join("profile-runtime/codex/skills/harness-ops")
            .exists(),
        "a second top-level harness skill must not compete with /agent-env-codex"
    );
    assert!(
        !root.join("agent-harness-skills").exists(),
        "split status/full/restricted/toggle skill products must not return"
    );

    let skill = fs::read_to_string(root.join("agent-skills/agent-env-codex/SKILL.md")).unwrap();
    assert!(skill.contains("references/github-execution-policy.md"));
    assert!(skill.contains("references/github-org-and-ccboard.md"));
    assert!(skill.contains("references/bunx-and-github-ssh.md"));
    assert!(skill.contains("never cherry-pick"));
    assert!(skill.contains("Never change `/permissions`"));
    assert!(skill.contains("Never invoke raw `git`"));
    assert!(skill.contains("Never add macOS or Windows GitHub Actions jobs"));

    let github_policy = fs::read_to_string(
        root.join("agent-skills/agent-env-codex/references/github-execution-policy.md"),
    )
    .unwrap();
    for required in [
        "Strict upgrade only",
        "Never cherry-pick",
        "No stranded commits",
        "Unfinished-work closure",
        "Permission integrity",
        "Meta worktree authority",
        "Personal and organization SSH proof",
        "Linux-only automation",
        "Protected trunks and disposable task state",
        "Non-destructive fork sync",
        "Branch/origin/worktree convergence",
    ] {
        assert!(github_policy.contains(required), "missing {required}");
    }
    assert!(github_policy.contains("rtk meta git"));
    assert!(github_policy.contains("main`, `master`, or `develop"));
    assert!(github_policy.contains("enable auto-merge"));

    let bunx_ssh = fs::read_to_string(
        root.join("agent-skills/agent-env-codex/references/bunx-and-github-ssh.md"),
    )
    .unwrap();
    for required in [
        "npm install",
        "bun install",
        "npx <package>",
        "bunx <package>",
        "bunx ruv-swarm/claude-flow@alpha",
        "drdave-flexnetos",
        "user/memberships/orgs/FlexNetOS",
        "git ls-remote git@github.com:FlexNetOS/envctl.git HEAD",
    ] {
        assert!(bunx_ssh.contains(required), "missing {required}");
    }
    let bun_policy = Command::new("python3")
        .arg(root.join("agent-skills/agent-env-codex/scripts/check-bun-command-policy.py"))
        .arg(&root)
        .status()
        .expect("run Bun/Bunx skill command policy");
    assert!(bun_policy.success(), "skill command policy must pass");

    let org_ccboard = fs::read_to_string(
        root.join("agent-skills/agent-env-codex/references/github-org-and-ccboard.md"),
    )
    .unwrap();
    for required in [
        "FlexNetOS organization surface matrix",
        "Secrets, variables, environments",
        "Webhooks and deploy keys",
        "GitHub Apps",
        "Code security and quality",
        "Custom properties",
        "ccboard and Claude/Codex implementation path",
        "Codex is partially wired, not absent",
        "DataStore::scan_third_party_sessions",
        "ccbrain-session-stop.sh",
        "codex-harness-claude-bridge",
        "Yazelix already owns the installed ccboard pane",
    ] {
        assert!(org_ccboard.contains(required), "missing {required}");
    }

    let workflows = root.join(".github/workflows");
    for entry in fs::read_dir(workflows).unwrap().flatten() {
        let path = entry.path();
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let workflow = fs::read_to_string(&path).unwrap().to_ascii_lowercase();
        assert!(
            !workflow.contains("macos-") && !workflow.contains("windows-"),
            "{} must remain Linux-only",
            path.display()
        );
    }

    let library = fs::read_to_string(root.join("home/agent-env/codex-harness/src/lib.rs")).unwrap();
    for required in [
        "SESSION_CAPABILITIES",
        "session_capability_status",
        "set_session_capability",
        "set_session_capability_preset",
        "CODEX_THREAD_ID",
    ] {
        assert!(library.contains(required), "missing {required}");
    }
}

#[test]
fn persistent_runner_jobs_have_isolated_cargo_targets() {
    let root = envctl_root();
    assert!(
        !root.join(".github/workflows/ci.yml").exists(),
        "the retired CI workflow must not become an active runner path"
    );
    let workflow = fs::read_to_string(root.join(".github/workflows_disabled/ci.yml")).unwrap();
    assert!(workflow.contains("CARGO_TARGET_DIR"));
    assert!(workflow.contains("RUNNER_TEMP"));
    assert!(workflow.contains("GITHUB_RUN_ID"));
    assert!(workflow.contains("GITHUB_RUN_ATTEMPT"));
    assert!(workflow.contains("GITHUB_JOB"));
    assert!(workflow.contains("GITHUB_ENV"));
    assert_eq!(
        workflow
            .matches("isolate Cargo target for this persistent-runner job")
            .count(),
        6,
        "every Cargo-using job must isolate its target directory"
    );
}

#[test]
fn agent_env_codex_requires_latest_yazelix_convergence_and_plugin_ownership() {
    let root = envctl_root();
    let skill = fs::read_to_string(root.join("agent-skills/agent-env-codex/SKILL.md")).unwrap();
    for required in [
        "references/yazelix-cli-plugin-policy.md",
        "A toggle may be off",
        "latest available Nix/Yazelix/fenix/Bun-owned binaries",
        "yzx update",
        "yazelix-yazi-assets",
        "Never leave completed or idle subagents running.",
    ] {
        assert!(
            skill.contains(required),
            "missing skill contract: {required}"
        );
    }

    let policy = fs::read_to_string(
        root.join("agent-skills/agent-env-codex/references/yazelix-cli-plugin-policy.md"),
    )
    .unwrap();
    for required in [
        "Do not invent a `yzx sync` command",
        "yzx update local_source",
        "yzx update upstream",
        "yzx update home_manager",
        "yzx doctor --fix-plan --json",
        "/home/flexnetos/meta/src/yazelix-yazi-assets",
        "yazelix_helix_cogs_noop_wt",
        "yazelix-helix",
        "yazelix_pane_orchestrator.wasm",
        "yzpp.wasm",
        "zjstatus.wasm",
    ] {
        assert!(
            policy.contains(required),
            "missing Yazelix policy: {required}"
        );
    }

    let prompt = fs::read_to_string(
        root.join(".codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md"),
    )
    .unwrap();
    for required in [
        "Mandatory-task, latest-toolchain, and Yazelix convergence controller",
        "The word `optional` means mandatory when attached to work",
        "latest available profile-owned toolchain",
        "plugin and add-on source/package/manifest authority",
        "empty harness-owned roster",
        "references/bunx-and-github-ssh.md",
        "references/github-execution-policy.md",
        "references/github-org-and-ccboard.md",
        "references/yazelix-cli-plugin-policy.md",
        "scripts/check-bun-command-policy.py",
        "scripts/check-yazelix-contract.py",
    ] {
        assert!(
            prompt.contains(required),
            "missing prompt contract: {required}"
        );
    }
    assert!(!prompt.contains("`rtk git ...`"));
    assert!(!prompt.contains("/bin/rtk git status"));

    for retired in [
        ".claude/prompts/prompt:claude-code-agent-env-ultraplan.prompt.md",
        ".claude/skills/agent-env-claude",
    ] {
        assert!(
            !root.join(retired).exists(),
            "retired Claude mirror must remain absent: {retired}"
        );
    }
}

#[test]
fn provider_and_rtk_routes_preserve_supervised_execution() {
    let root = envctl_root();
    let providers =
        fs::read_to_string(root.join("home/agent-env/codex-harness/config/policy/providers.toml"))
            .unwrap();
    for required in [
        "--safe-mode",
        "--tools \\\"\\\"",
        "--strict-mcp-config",
        "--disable-slash-commands",
        "--no-chrome",
        "--no-session-persistence",
    ] {
        assert!(providers.contains(required), "missing {required}");
    }
    assert!(!providers.contains("--permission-mode plan"));

    for name in ["no-nested-agents.rules", "parallel-runner.rules"] {
        let harness =
            fs::read_to_string(root.join("home/agent-env/codex-harness/rules").join(name)).unwrap();
        assert!(harness.contains("pattern = [\"rtk\", \"codex\""));
        assert!(harness.contains("pattern = [\"rtk\", \"claude\""));
    }
    assert!(!root.join("profile-runtime/codex/rules").exists());
}
