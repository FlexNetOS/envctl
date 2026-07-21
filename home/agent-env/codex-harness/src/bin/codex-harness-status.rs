#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::{
    active_job_records, audit_value, bad_behavior_counts, count_lines, last_deny_summary,
    ledger_dir, model_router_ready, nix_verify_value, project_root, routed_tool_basename,
    session_capability_status, state_dir, tracked_full_access_policy_granted,
    USER_FULL_ACCESS_DECISION_ID,
};
use serde_json::Value;
use std::env;
use std::fs;
use std::process::Command;

fn capability_enabled(status: &Value, capability: &str) -> bool {
    status
        .pointer(&format!("/capabilities/{capability}/enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

fn active_model() -> String {
    env::var("CODEX_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            fs::read_to_string("/run/user/1001/yazelix/profile-runtime/codex/config.toml")
                .ok()
                .and_then(|text| toml::from_str::<toml::Value>(&text).ok())
                .and_then(|config| config.get("model")?.as_str().map(str::to_string))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() -> Result<()> {
    let jobs = active_job_records().unwrap_or_default();
    let nix = nix_verify_value();
    let audit = audit_value();
    let mut codex_jobs = 0usize;
    let mut claude_jobs = 0usize;
    let mut local_model_jobs = 0usize;
    for job in &jobs {
        match routed_tool_basename(&job.argv).as_str() {
            "codex" => codex_jobs += 1,
            "claude" => claude_jobs += 1,
            "ollama" | "lms" | "lmstudio" => local_model_jobs += 1,
            _ => {}
        }
    }
    let branch = Command::new("git")
        .args([
            "-C",
            &project_root().display().to_string(),
            "branch",
            "--show-current",
        ])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let budget_events = count_lines(&ledger_dir().join("budget.jsonl")).unwrap_or(0);
    let last_deny = last_deny_summary().unwrap_or_else(|_| "unavailable".to_string());
    let bad_counts = bad_behavior_counts().unwrap_or_default();
    let bad_total: u64 = bad_counts.values().copied().sum();
    let bad_summary = if bad_counts.is_empty() {
        "none".to_string()
    } else {
        bad_counts
            .iter()
            .map(|(kind, count)| format!("{kind}:{count}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let codex_path = nix
        .get("codex_path")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let unresolved = state_dir().join("unresolved-decision").exists();
    let session_status = session_capability_status();
    let session_state_valid = session_status
        .get("state_valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let permission_profile = session_status
        .get("permission_profile_signal")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let network = capability_enabled(&session_status, "network");
    let external_providers = capability_enabled(&session_status, "external_providers");
    let local_models = capability_enabled(&session_status, "local_models");
    let browser_computer = capability_enabled(&session_status, "browser_computer");
    let github_mutation = capability_enabled(&session_status, "github_mutation");
    let subagents = capability_enabled(&session_status, "subagents");
    let background_jobs = capability_enabled(&session_status, "background_jobs");
    let model = active_model();
    println!(
        "codex-harness status configured_model={} live_model=unknown-use-/model permission_profile_signal={} live_permissions=unknown-use-/permissions project={} branch={} codex_path={} nix_owned={} operator_full_access_intent_recorded={} operator_decision_id={} session_state_valid={} network={} external_providers={} local_models={} subagents={} background_jobs={} openrouter={} claude_bridge={} browser_computer={} github_full_access={} active_native_subagents=unknown-use-/agent active_background_jobs={} active_codex_child_sessions={} active_claude_child_sessions={} active_local_model_jobs={} budget_events={} ledger_ok={} model_router_ready={} open_decisions={} bad_behavior_total={} bad_behavior_counters={} last_deny={}",
        model,
        permission_profile,
        project_root().display(),
        branch,
        codex_path,
        nix.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        tracked_full_access_policy_granted(),
        if tracked_full_access_policy_granted() { USER_FULL_ACCESS_DECISION_ID } else { "none" },
        session_state_valid,
        enabled_label(network),
        enabled_label(external_providers),
        enabled_label(local_models),
        enabled_label(subagents),
        enabled_label(background_jobs),
        enabled_label(external_providers && network),
        enabled_label(external_providers && network),
        enabled_label(browser_computer),
        enabled_label(github_mutation && network),
        jobs.len(),
        codex_jobs,
        claude_jobs,
        local_model_jobs,
        budget_events,
        audit.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        model_router_ready(),
        unresolved,
        bad_total,
        bad_summary,
        last_deny
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn capability_status_is_read_from_effective_map() {
        let status = json!({
            "capabilities": {
                "network": {"enabled": false},
                "external_providers": {"enabled": true},
            }
        });
        assert!(!capability_enabled(&status, "network"));
        assert!(capability_enabled(&status, "external_providers"));
        assert!(!capability_enabled(&status, "missing"));
    }
}
