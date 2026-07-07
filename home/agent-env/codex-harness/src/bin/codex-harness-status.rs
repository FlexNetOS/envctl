#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::{
    active_job_records, audit_value, count_lines, last_deny_summary, ledger_dir, nix_verify_value,
    project_root,
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
    let codex_path = nix
        .get("codex_path")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let unresolved = Path::new(
        "/home/flexnetos/lifeos/src/envctl/home/agent-env/codex-harness/state/unresolved-decision",
    )
    .exists();
    println!(
        "codex-harness status model=gpt-5.5 permission_profile=envctl-harness project={} branch={} codex_path={} nix_owned={} active_subagents=0 active_background_jobs={} active_codex_child_sessions={} active_claude_child_sessions={} active_local_model_jobs={} budget_events={} ledger_ok={} open_decisions={} last_deny={}",
        project_root().display(),
        branch,
        codex_path,
        nix.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        jobs.len(),
        codex_jobs,
        claude_jobs,
        local_model_jobs,
        budget_events,
        audit.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        unresolved,
        last_deny
    );
    Ok(())
}
