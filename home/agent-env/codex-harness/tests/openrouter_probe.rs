use codex_harness::{openrouter_model_catalog_summary, openrouter_wire_compatibility_summary};
use serde_json::json;

#[test]
fn openrouter_catalog_summary_proves_default_and_major_provider_families() {
    let catalog = json!({
        "data": [
            {"id": "openai/gpt-5.2"},
            {"id": "anthropic/claude-sonnet-4.5"},
            {"id": "tencent/hy3:free"}
        ]
    });

    let summary = openrouter_model_catalog_summary(&catalog, "tencent/hy3:free");

    assert_eq!(summary["model_count"], 3);
    assert_eq!(summary["has_openai_models"], true);
    assert_eq!(summary["has_anthropic_models"], true);
    assert_eq!(summary["has_target_model"], true);
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
