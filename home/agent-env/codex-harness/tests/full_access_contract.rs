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
    for entry in fs::read_dir(config_root.join("agents")).unwrap() {
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
        "harness-status/SKILL.md",
        "harness-full/SKILL.md",
        "harness-restricted/SKILL.md",
        "harness-toggle/SKILL.md",
    ] {
        assert!(skill_root.join(skill).is_file(), "{skill}");
    }
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
