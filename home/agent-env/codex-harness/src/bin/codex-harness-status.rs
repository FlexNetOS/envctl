#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::{
    USER_FULL_ACCESS_DECISION_ID, active_job_records, audit_value, bad_behavior_counts,
    count_lines, full_access_granted, last_deny_summary, ledger_dir, model_route_for_task,
    model_router_ready, nix_verify_value, project_root, state_dir,
};
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    let jobs = active_job_records().unwrap_or_default();
    let nix = nix_verify_value();
    let audit = audit_value();
    let mut codex_jobs = 0usize;
    let mut claude_jobs = 0usize;
    let mut local_model_jobs = 0usize;
    for job in &jobs {
        let first = job.argv.first().map(String::as_str).unwrap_or_default();
        let bin = Path::new(first)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(first);
        match bin {
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
    let default_route = model_route_for_task("professional implementation");
    let default_model = default_route
        .get("model")
        .and_then(|value| value.as_str())
        .unwrap_or("gpt-5.6-terra");
    println!(
        "codex-harness status model={} permission_profile=danger-full-access project={} branch={} codex_path={} nix_owned={} full_access_granted={} full_access_decision_id={} openrouter=enabled claude_bridge=enabled browser_computer=enabled github_full_access=enabled active_subagents=0 active_background_jobs={} active_codex_child_sessions={} active_claude_child_sessions={} active_local_model_jobs={} budget_events={} ledger_ok={} model_router_ready={} open_decisions={} bad_behavior_total={} bad_behavior_counters={} last_deny={}",
        default_model,
        project_root().display(),
        branch,
        codex_path,
        nix.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        full_access_granted(),
        if full_access_granted() {
            USER_FULL_ACCESS_DECISION_ID
        } else {
            "none"
        },
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
