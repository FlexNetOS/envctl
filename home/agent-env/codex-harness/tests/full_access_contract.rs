#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

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
    assert!(
        !library.contains("pub fn full_access_granted()"),
        "tracked operator policy must not masquerade as effective live permissions"
    );
    let capability_body = library
        .split("pub fn session_capability_enabled")
        .nth(1)
        .and_then(|tail| tail.split("pub fn").next())
        .expect("session_capability_enabled body");
    assert!(
        !capability_body.contains("tracked_full_access_policy_granted"),
        "optional routing switches must not create a duplicate permission gate"
    );
}

#[test]
fn every_routeable_profile_is_full_access_without_retired_hooks() {
    let root = envctl_root();
    let config_dir = root.join("home/.codex");
    let active_source: toml::Value = fs::read_to_string(config_dir.join("config.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let source_features = active_source
        .get("features")
        .and_then(toml::Value::as_table)
        .unwrap();
    assert_eq!(
        source_features
            .get("shell_zsh_fork")
            .and_then(toml::Value::as_bool),
        Some(false),
        "Codex zsh fork path silently exits every shell command in the active runtime"
    );
    assert_eq!(
        source_features
            .get("unified_exec_zsh_fork")
            .and_then(toml::Value::as_bool),
        Some(false),
        "unified-exec zsh fork path silently exits every shell command"
    );
    assert_eq!(
        source_features
            .get("image_generation")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert!(
        source_features.get("imagegenext").is_none(),
        "retired imagegenext emits an error item on every new Codex turn"
    );
    for entry in fs::read_dir(config_dir).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.starts_with("envctl-") || !name.ends_with(".config.toml") {
            continue;
        }
        let config = parse_toml(&path);
        if name == "envctl-yolo-breakglass-disabled.config.toml" {
            assert_ne!(
                string_at(&config, "sandbox_mode"),
                Some("danger-full-access"),
                "{name} must remain an intentionally disabled bypass trap"
            );
            continue;
        }
        if let Some(model_catalog) = string_at(&config, "model_catalog_json") {
            assert_eq!(
                model_catalog, "/home/flexnetos/.codex/model-catalog.json",
                "{name} bypasses the active Codex model catalog owner"
            );
        }
        assert_eq!(
            string_at(&config, "approval_policy"),
            Some("never"),
            "{name}"
        );
        assert_eq!(
            string_at(&config, "sandbox_mode"),
            Some("danger-full-access"),
            "{name}"
        );
        assert_eq!(
            string_at(&config, "default_permissions"),
            Some(":danger-full-access"),
            "{name}"
        );
        if let Some(features) = config.get("features") {
            assert_ne!(
                features.get("hooks").and_then(toml::Value::as_bool),
                Some(true),
                "{name} re-enables retired hooks"
            );
        }
    }
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

    let active_agents = root.join("home/.codex/agents");
    for entry in fs::read_dir(&active_agents).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        let agent = parse_toml(&path);
        assert_eq!(
            string_at(&agent, "sandbox_mode"),
            Some("danger-full-access"),
            "{}",
            path.display()
        );
    }
}

#[test]
fn routeable_surfaces_use_sol_terra_luna_instead_of_gpt55() {
    let root = envctl_root();
    let config_root = root.join("home/.codex");
    for entry in fs::read_dir(&config_root).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with("envctl-") && name.ends_with(".config.toml") {
            let text = fs::read_to_string(&path).unwrap();
            assert!(
                !text.contains("model = \"gpt-5.5\""),
                "{name} must not route work to GPT-5.5"
            );
        }
    }
    for agents_root in [config_root.join("agents")] {
        for entry in fs::read_dir(agents_root).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|value| value.to_str()) != Some("toml") {
                continue;
            }
            let agent = parse_toml(&path);
            let model = string_at(&agent, "model").expect("agent model");
            assert!(
                matches!(model, "gpt-5.6-sol" | "gpt-5.6-terra" | "gpt-5.6-luna"),
                "{} routes to {model}",
                path.display()
            );
        }
    }
    for entry in fs::read_dir(root.join(".codex/agents")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        let agent = parse_toml(&path);
        assert_ne!(
            string_at(&agent, "model"),
            Some("gpt-5.5"),
            "{} must not route to GPT-5.5",
            path.display()
        );
    }
    let providers =
        fs::read_to_string(root.join("home/agent-env/codex-harness/config/policy/providers.toml"))
            .unwrap();
    assert!(providers.contains("primary_model = \"gpt-5.6-sol\""));
    let task_matrix = fs::read_to_string(
        root.join("home/agent-env/codex-harness/model-catalog/model-task-matrix.toml"),
    )
    .unwrap();
    assert!(!task_matrix.contains("model = \"gpt-5.5\""));
    assert!(!task_matrix.contains("profile = \"envctl-gpt55"));

    let top_level = parse_toml(&root.join("home/.codex/config.toml"));
    assert!(
        top_level
            .get("notice")
            .and_then(|value| value.get("model_migrations"))
            .and_then(|value| value.get("gpt-5.5"))
            .is_none(),
        "top-level config must not retain a GPT-5.5 migration route"
    );
    for relative in [
        "home/.codex/model-catalog.json",
        "home/agent-env/codex-harness/model-catalog/model-catalog.json",
    ] {
        let catalog: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(relative)).unwrap()).unwrap();
        let slugs = catalog
            .get("models")
            .and_then(serde_json::Value::as_array)
            .expect("model catalog array")
            .iter()
            .filter_map(|model| model.get("slug").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>();
        for forbidden in ["gpt-5.5", "gpt-5.5-pro", "gpt-5.5-pro-extended"] {
            assert!(!slugs.contains(&forbidden), "{relative} routes {forbidden}");
        }
    }
    assert!(
        !root.join("home/.codex/models_cache.json").exists(),
        "tracked API model cache must not remain as a second route authority"
    );
    let access_matrix = fs::read_to_string(
        root.join("home/agent-env/codex-harness/model-catalog/model-access-matrix.toml"),
    )
    .unwrap();
    for forbidden in ["gpt-5.5", "gpt-5.5-pro", "gpt-5.5-pro-extended"] {
        assert!(
            !access_matrix.contains(forbidden),
            "model access matrix routes {forbidden}"
        );
    }
}

#[test]
fn session_bootstrap_is_read_only_and_not_a_capability() {
    let root = envctl_root();
    let skill =
        fs::read_to_string(root.join("agent-harness-skills/skills/harness-init/SKILL.md")).unwrap();
    let runbook = fs::read_to_string(root.join("docs/runbook/README.md")).unwrap();
    let home_agents = fs::read_to_string(root.join("home/AGENTS.md")).unwrap();
    for required in [
        "git-kb list --path context/ --json",
        "icm --read-only wake-up --max-tokens 200",
        "rtk meta git",
        "rtk meta exec -- git",
        "--include <repo>",
        "rtk init --show",
    ] {
        assert!(
            skill.contains(required),
            "bootstrap skill missing {required}"
        );
    }
    for forbidden_auto_init in [
        "Never run `git-kb init`",
        "`grit init`",
        "`icm init`",
        "`meta init`",
        "mutating `rtk init`",
    ] {
        assert!(
            skill.contains(forbidden_auto_init),
            "bootstrap skill must forbid automatic {forbidden_auto_init}"
        );
    }
    assert!(runbook.contains("$harness-init"));
    assert!(home_agents.contains("Session tool bootstrap"));
    for forbidden_capability in [
        "gitkb_init",
        "grit_init",
        "icm_init",
        "meta_init",
        "rtk_init",
    ] {
        assert!(
            !codex_harness::SESSION_CAPABILITIES.contains(&forbidden_capability),
            "{forbidden_capability} must not become a hidden mutation toggle"
        );
    }
}

#[test]
fn supervised_native_provider_routes_use_profile_rtk() {
    let root = envctl_root();
    let library = fs::read_to_string(root.join("home/agent-env/codex-harness/src/lib.rs")).unwrap();
    assert!(library.contains("profile_rtk_argv(\"codex\""));
    assert!(library.contains("profile_rtk_command(\"codex\""));
    assert!(library.contains("profile_rtk_argv(\"claude\""));
    assert!(library.contains("profile_rtk_command(\"claude\""));
    assert!(library.contains("YAZELIX_PROFILE_ROOT"));
}

#[test]
fn native_rule_mirrors_allow_the_operator_frontdoor() {
    let root = envctl_root();
    let rule_paths = [
        "home/.codex/rules/no-yolo.rules",
        "home/agent-env/codex-harness/rules/no-yolo.rules",
    ];
    let mut previous = None;
    for relative in rule_paths {
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
        if let Some(expected) = previous {
            assert_eq!(text, expected, "native rule mirrors diverged");
        }
        previous = Some(text);
    }
}

#[test]
fn session_control_skills_are_global_and_cover_every_toggle() {
    let root = envctl_root();
    let config = fs::read_to_string(root.join("agent-harness-controls/agent-env.yaml")).unwrap();
    assert!(config.contains("scope: global"));
    assert!(config.contains("agent: codex"));
    assert!(config.contains("source: ../agent-harness-skills"));

    let skill_root = root.join("agent-harness-skills/skills");
    for skill in [
        "harness-session/SKILL.md",
        "harness-init/SKILL.md",
        "harness-status/SKILL.md",
        "harness-full/SKILL.md",
        "harness-restricted/SKILL.md",
        "harness-toggle/SKILL.md",
    ] {
        assert!(skill_root.join(skill).is_file(), "{skill}");
    }
    let session = fs::read_to_string(skill_root.join("harness-session/SKILL.md")).unwrap();
    for required in [
        "CODEX-GPT-HARNESS",
        ".codex/prompts/prompt:codex-gpt-harness.prompt.md",
        "$harness-init",
        "$harness-status",
        "$harness-full",
        "$harness-restricted",
        "$harness-toggle",
        "/permissions",
        "Yazelix/Nix profile-owned frontdoors",
        "Archive before changing existing files",
        "Never read, print, paste, or commit secrets",
        "Sol/Terra/Luna",
        "Do not restore GPT-5.5",
        "git status --short --branch",
        "gh pr list --state open",
    ] {
        assert!(session.contains(required), "{required}");
    }
    assert!(session.contains("Do not keep growing those prompt files as the primary control plane"));
    let toggle = fs::read_to_string(skill_root.join("harness-toggle/SKILL.md")).unwrap();
    assert!(
        toggle.contains("/home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/Cargo.toml")
    );
    for capability in [
        "external_providers",
        "local_models",
        "network",
        "github_mutation",
        "browser_computer",
        "subagents",
        "background_jobs",
    ] {
        assert!(toggle.contains(capability), "{capability}");
    }
    assert!(toggle.contains("/permissions"));
    assert!(toggle.contains("actual Codex sandbox"));
}

#[test]
fn provider_contracts_do_not_restore_permission_or_path_blockers() {
    let root = envctl_root();
    let library = fs::read_to_string(root.join("home/agent-env/codex-harness/src/lib.rs")).unwrap();
    let providers =
        fs::read_to_string(root.join("home/agent-env/codex-harness/config/policy/providers.toml"))
            .unwrap();
    let runner = fs::read_to_string(
        root.join("home/agent-env/codex-harness/src/bin/codex-harness-runner.rs"),
    )
    .unwrap();
    assert!(library.contains("\"safety_mode\": \"safe-mode-no-tools-strict-empty-mcp\""));
    for required_flag in [
        "--safe-mode",
        "--tools",
        "--strict-mcp-config",
        "--disable-slash-commands",
        "--no-chrome",
        "--no-session-persistence",
    ] {
        assert!(library.contains(required_flag), "{required_flag}");
    }
    assert!(!library.contains("\"permission_mode\": \"bypassPermissions\""));
    assert!(!providers.contains("bypassPermissions"));
    assert!(providers.contains("--safe-mode"));
    assert!(providers.contains("--tools \\\"\\\""));
    assert!(providers.contains("--strict-mcp-config"));
    assert!(library.contains("pub fn spawn_claude_run"));
    assert!(library.contains("\"execution_ready\": execution_ready"));
    assert!(library.contains("\"provider-local-ollama\""));
    assert!(
        runner.contains("if result.get(\"ok\").and_then(serde_json::Value::as_bool) != Some(true)"),
        "provider probes must return a nonzero process status when execution readiness is false"
    );

    for relative in [
        "home/.codex/rules/provider-routing.rules",
        "home/.codex/rules/github.rules",
        "home/agent-env/codex-harness/rules/provider-routing.rules",
        "home/agent-env/codex-harness/rules/github.rules",
    ] {
        let rules = fs::read_to_string(root.join(relative)).unwrap();
        assert!(
            !rules.contains("/home/flexnetos/meta/src/envctl"),
            "{relative} contains a canonical-checkout-only executable path"
        );
    }
}
