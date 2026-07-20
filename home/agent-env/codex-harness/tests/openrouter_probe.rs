use codex_harness::{
    openrouter_model_catalog_summary, openrouter_probe_prompt, openrouter_responses_summary,
    openrouter_transport_summary, openrouter_wire_compatibility_summary,
};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const HY3: &str = "tencent/hy3:free";

fn envctl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("harness crate path has envctl ancestor")
        .to_path_buf()
}

fn read_json(relative: &str) -> Value {
    let path = envctl_root().join(relative);
    serde_json::from_str(
        &fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn openrouter_catalog_summary_proves_default_and_major_provider_families() {
    let catalog = json!({
        "data": [
            {"id": "openai/gpt-5.2"},
            {"id": "anthropic/claude-sonnet-4.5"},
            {"id": "tencent/hy3:free"}
        ]
    });

    let summary = openrouter_model_catalog_summary(&catalog, HY3);

    assert_eq!(summary["model_count"], 3);
    assert_eq!(summary["has_openai_models"], true);
    assert_eq!(summary["has_anthropic_models"], true);
    assert_eq!(summary["has_target_model"], true);
}

#[test]
fn openrouter_catalog_summary_rejects_missing_target_model() {
    let summary =
        openrouter_model_catalog_summary(&json!({"data": [{"id": "openai/gpt-5.2"}]}), HY3);
    assert_eq!(summary["has_target_model"], false);
    assert_eq!(summary["target_model"], HY3);
}

#[test]
fn tracked_catalogs_and_profiles_expose_current_hy3_contract() {
    for relative in [
        "home/.codex/model-catalog.json",
        "home/agent-env/codex-harness/model-catalog/model-catalog.json",
    ] {
        let catalog = read_json(relative);
        let model = catalog["models"]
            .as_array()
            .expect("models array")
            .iter()
            .find(|model| model["slug"] == HY3)
            .unwrap_or_else(|| panic!("{relative} lacks {HY3}"));
        assert_eq!(model["provider"], "openrouter", "{relative}");
        assert_eq!(model["context_window"], 262_144, "{relative}");
        assert_eq!(model["max_context_window"], 262_144, "{relative}");
        assert_eq!(model["free_route_expires_on"], "2026-07-21", "{relative}");
        let efforts = model["supported_reasoning_levels"]
            .as_array()
            .expect("reasoning levels")
            .iter()
            .filter_map(|level| level["effort"].as_str())
            .collect::<Vec<_>>();
        assert_eq!(efforts, ["low", "high"], "{relative}");
    }

    for relative in [
        "home/.codex/envctl-openrouter.config.toml",
        "home/.codex/envctl-openrouter-gpt.config.toml",
    ] {
        let path = envctl_root().join(relative);
        let profile: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(profile["model"].as_str(), Some(HY3), "{relative}");
        assert_eq!(
            profile["model_provider"].as_str(),
            Some("openrouter"),
            "{relative}"
        );
        let provider = &profile["model_providers"]["openrouter"];
        assert_eq!(
            provider["base_url"].as_str(),
            Some("https://openrouter.ai/api/v1"),
            "{relative}"
        );
        assert_eq!(
            provider["env_key"].as_str(),
            Some("OPENROUTER_API_KEY"),
            "{relative}"
        );
        assert_eq!(
            provider["wire_api"].as_str(),
            Some("responses"),
            "{relative}"
        );
    }
}

#[test]
fn openrouter_response_rejects_http_success_with_api_error() {
    let summary = openrouter_responses_summary(
        "200",
        0,
        &json!({"status": "completed", "error": {"message": "denied"}, "output": []}),
        None,
    );
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["api_error"], true);
}

#[test]
fn openrouter_response_rejects_empty_completed_output() {
    let summary = openrouter_responses_summary(
        "200",
        0,
        &json!({"status": "completed", "error": null, "output": []}),
        None,
    );
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["output_nonempty"], false);
}

#[test]
fn openrouter_response_accepts_completed_output_and_optional_marker() {
    let body = json!({
        "status": "completed",
        "error": null,
        "output": [{
            "type": "message",
            "content": [{"type": "output_text", "text": "HY3_LIVE_OK"}]
        }]
    });
    let summary = openrouter_responses_summary("200", 0, &body, Some("HY3_LIVE_OK"));
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["completed"], true);
    assert_eq!(summary["marker_present"], true);
    assert_eq!(summary["output_len"], 11);
    assert!(
        summary.get("output").is_none(),
        "proof must not retain raw output"
    );
}

#[test]
fn openrouter_response_rejects_missing_expected_marker() {
    let body = json!({"status": "completed", "output_text": "different output"});
    let summary = openrouter_responses_summary("200", 0, &body, Some("HY3_LIVE_OK"));
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["marker_present"], false);
}

#[test]
fn openrouter_transport_summary_never_persists_raw_provider_output_or_stderr() {
    let sentinel = "sk-or-v1-OPENROUTER_SECRET_SENTINEL";
    let body = json!({
        "status": "completed",
        "error": null,
        "output_text": sentinel
    })
    .to_string();
    let summary = openrouter_transport_summary("200", 0, &body, sentinel, None);
    let persisted = serde_json::to_string(&summary).unwrap();

    assert_eq!(summary["result"]["ok"], true);
    assert_eq!(summary["response_body_len"], body.len());
    assert_eq!(summary["stderr_len"], sentinel.len());
    assert!(!persisted.contains(sentinel));
    assert!(summary.get("response_redacted_preview").is_none());
    assert!(summary.get("stderr_redacted").is_none());
}

#[test]
fn openrouter_default_probe_requires_a_unique_marker_but_custom_prompts_do_not() {
    let (default_prompt, marker) = openrouter_probe_prompt(None);
    assert_eq!(marker, Some("HY3_OPENROUTER_LIVE_OK"));
    assert!(default_prompt.contains(marker.unwrap()));

    let (custom_prompt, marker) = openrouter_probe_prompt(Some("explain the result"));
    assert_eq!(custom_prompt, "explain the result");
    assert_eq!(marker, None);
}

#[test]
fn openrouter_wire_summary_proves_responses_api_and_chat_fallback() {
    let openapi = json!({
        "paths": {
            "/responses": {"post": {"operationId": "responses_create"}},
            "/chat/completions": {"post": {"operationId": "chat_create"}}
        }
    });

    let summary = openrouter_wire_compatibility_summary(&openapi);

    assert_eq!(summary["responses_api_documented"], true);
    assert_eq!(summary["chat_completions_documented"], true);
    assert_eq!(summary["direct_responses_wire_compatible"], true);
    assert_eq!(summary["chat_completion_fallback_available"], true);
}
