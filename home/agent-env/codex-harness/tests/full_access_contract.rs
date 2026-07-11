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
}

#[test]
fn every_routeable_profile_is_full_access_without_retired_hooks() {
    let root = envctl_root();
    let config_dir = root.join("home/.codex");
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
fn routeable_models_are_sol_terra_luna_or_explicit_fallbacks() {
    let root = envctl_root();
    let config_dir = root.join("home/.codex");
    for entry in fs::read_dir(config_dir).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.starts_with("envctl-") || !name.ends_with(".config.toml") {
            continue;
        }
        let config = parse_toml(&path);
        assert_ne!(
            string_at(&config, "model"),
            Some("gpt-5.5"),
            "{name} retains a routeable GPT-5.5 assignment"
        );
    }

    let main_config = parse_toml(&root.join("home/.codex/config.toml"));
    assert_eq!(
        string_at(&main_config, "model"),
        Some("gpt-5.6-terra"),
        "Terra must be the balanced default lane"
    );

    let agents = [
        ("rust-harness-engineer.toml", "gpt-5.6-sol"),
        ("security-reviewer.toml", "gpt-5.6-sol"),
        ("verifier.toml", "gpt-5.6-sol"),
        ("researcher.toml", "gpt-5.6-terra"),
        ("explorer.toml", "gpt-5.6-luna"),
    ];
    for (name, expected) in agents {
        let agent = parse_toml(&root.join("home/.codex/agents").join(name));
        assert_eq!(string_at(&agent, "model"), Some(expected), "{name}");
    }
}

#[test]
fn retired_roots_and_tracked_model_cache_are_absent() {
    let root = envctl_root();
    assert!(
        !root.join("home/.codex/models_cache.json").exists(),
        "tracked models_cache.json must not be a secondary authority"
    );

    let library = fs::read_to_string(root.join("home/agent-env/codex-harness/src/lib.rs")).unwrap();
    assert!(!library.contains("/home/flexnetos/lifeos/src/envctl"));
    assert!(library.contains("/home/flexnetos/meta/src/envctl/home"));

    let config = fs::read_to_string(root.join("home/.codex/config.toml")).unwrap();
    assert!(!config.contains("[projects.\"/home/flexnetos/lifeos"));
    assert!(!config.contains("[projects.\"/home/flexnetos/FlexNetOS"));
    assert!(!config.contains("\"/home/flexnetos/lifeos/src/envctl\""));
    assert!(config.contains("[projects.\"/home/flexnetos/meta/src/envctl\"]"));
    assert!(config.contains("image_generation = true"));
    assert!(!config.contains("imagegenext"));

    let baseline =
        fs::read_to_string(root.join("manifest/components.d/codex-global-baseline.toml")).unwrap();
    assert!(
        !baseline.contains("'model = \"gpt-5.5\"'"),
        "meta-local generated runtime must not route GPT-5.5"
    );
    assert!(baseline.contains("'model = \"gpt-5.6-terra\"'"));
    assert!(!baseline.contains("LIFEOS_ROOT"));
    assert!(baseline.contains("home/.codex/agents"));

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
