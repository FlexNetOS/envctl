#![forbid(unsafe_code)]

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptFinding {
    pub severity: String,
    pub category: String,
    pub pattern: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptReviewReport {
    pub ok: bool,
    pub prompt_path: Option<PathBuf>,
    pub line_count: usize,
    pub launch_flag_count: usize,
    pub missing_required_anchors: Vec<String>,
    pub missing_phase11_anchors: Vec<String>,
    pub forbidden_findings: Vec<PromptFinding>,
}

const LAUNCH_FLAG: &str = "--dangerously-bypass-approvals-and-sandbox";

const REQUIRED_ANCHORS: &[(&str, &str)] = &[
    ("variant_header", "FULL-ACCESS NO-SANDBOX VARIANT"),
    (
        "launch_command",
        "codex --dangerously-bypass-approvals-and-sandbox",
    ),
    ("approval_policy_never", "approval_policy = \"never\""),
    (
        "sandbox_danger_full_access",
        "sandbox_mode = \"danger-full-access\"",
    ),
    (
        "default_permissions_danger_full_access",
        "default_permissions = \":danger-full-access\"",
    ),
    (
        "full_local_filesystem_baseline",
        "full local filesystem access is the baseline",
    ),
    ("network_enabled", "network access is enabled"),
    ("sandboxing_skipped", "sandboxing is skipped"),
    (
        "no_request_permissions",
        "do not call `request_permissions`",
    ),
    (
        "phase11_full_access",
        "Phase 11 implementation must execute under full access/no sandbox",
    ),
];

const PHASE11_ANCHORS: &[(&str, &str)] = &[
    ("phase11_header", "PHASE 11"),
    (
        "full_access_verification_prompt",
        "codex exec --json with full-access verification prompt",
    ),
    ("subagent_drill_full_access", "Run real full-access drill"),
    (
        "browser_computer_full_access",
        "Under the full-access no-sandbox controller",
    ),
    (
        "github_guard_full_access",
        "GitHub mutation uses github guard and full-access context",
    ),
    (
        "failure_truth_mode",
        "record fail/unsupported/gap and continue repair under full access",
    ),
];

/// These strings are not merely historical words; in this full-access prompt they
/// are known failure-loop instructions that can make the 11-phase implementation
/// downgrade to read-only, wait on a gate, or stop instead of recording evidence.
const FORBIDDEN_BLOCKER_PATTERNS: &[(&str, &str)] = &[
    ("read_only_mode", "read-only"),
    ("read_only_mode", "Read-only"),
    ("read_only_mode", "READ-ONLY"),
    ("workspace_write_mode", "workspace-write"),
    ("read_only_sandbox", "sandbox_mode = \"read-only\""),
    (
        "workspace_write_sandbox",
        "sandbox_mode = \"workspace-write\"",
    ),
    ("limited_mode_command", "Stay in a limited mode"),
    ("limited_mode_command", "Stay read-only"),
    ("mutation_ban", "Do not edit files"),
    ("mutation_ban", "Do not create files"),
    ("mutation_ban", "No mutation before Phase 0 completes"),
    ("phase_stop", "Do not proceed to Phase"),
    (
        "subagent_stop",
        "If subagent execution is not available, stop",
    ),
    ("approval_blocker", "approval required"),
    ("approval_blocker", "approval question"),
    ("approval_blocker", "with approval"),
    ("approval_blocker", "review/approval"),
    ("approval_blocker", "unless approved"),
    ("approval_blocker", "until approved"),
    ("approval_blocker", "operator approves"),
    ("repo_write_ban", "No repo writes"),
    ("network_disabled", "network disabled"),
    ("network_disabled", "network remains disabled"),
    (
        "permission_request_enabled",
        "request_permissions_tool = true",
    ),
    (
        "approval_prompts_enabled",
        "exec_permission_approvals = true",
    ),
    ("phase0_budget_gate", "budget ceiling: ask operator"),
];

pub fn review_full_access_prompt_path(path: impl AsRef<Path>) -> Result<PromptReviewReport> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read prompt: {}", path.display()))?;
    let mut report = review_full_access_prompt_text(&text);
    report.prompt_path = Some(path.to_path_buf());
    Ok(report)
}

pub fn review_full_access_prompt_text(text: &str) -> PromptReviewReport {
    let missing_required_anchors = REQUIRED_ANCHORS
        .iter()
        .filter(|(_, needle)| !text.contains(needle))
        .map(|(name, _)| (*name).to_string())
        .collect::<Vec<_>>();
    let mut missing_phase11_anchors = PHASE11_ANCHORS
        .iter()
        .filter(|(_, needle)| !text.contains(needle))
        .map(|(name, _)| (*name).to_string())
        .collect::<Vec<_>>();
    if text.matches(LAUNCH_FLAG).count() < 3 {
        missing_phase11_anchors.push("launch_flag_count_ge_3".to_string());
    }
    let forbidden_findings = forbidden_findings(text);
    PromptReviewReport {
        ok: missing_required_anchors.is_empty()
            && missing_phase11_anchors.is_empty()
            && forbidden_findings.is_empty(),
        prompt_path: None,
        line_count: text.lines().count(),
        launch_flag_count: text.matches(LAUNCH_FLAG).count(),
        missing_required_anchors,
        missing_phase11_anchors,
        forbidden_findings,
    }
}

pub fn assert_prompt_review_ok(report: &PromptReviewReport) -> Result<()> {
    if report.ok {
        return Ok(());
    }
    Err(anyhow!(
        "full-access prompt review failed: missing_required={:?}; missing_phase11={:?}; forbidden={:?}",
        report.missing_required_anchors,
        report.missing_phase11_anchors,
        report.forbidden_findings
    ))
}

fn forbidden_findings(text: &str) -> Vec<PromptFinding> {
    let mut findings = Vec::new();
    for (category, pattern) in FORBIDDEN_BLOCKER_PATTERNS {
        for (line_index, line) in text.lines().enumerate() {
            if line.contains(pattern) && !is_allowed_exception(line, pattern) {
                findings.push(PromptFinding {
                    severity: "blocker".to_string(),
                    category: (*category).to_string(),
                    pattern: (*pattern).to_string(),
                    line: line_index + 1,
                    snippet: line.trim().chars().take(240).collect(),
                });
            }
        }
    }
    findings
}

fn is_allowed_exception(line: &str, pattern: &str) -> bool {
    // `--ask-for-approval never` is a required full-access/no-extra-gate flag,
    // not a request to ask for approval.
    if line.contains("--ask-for-approval never") {
        return true;
    }
    // Quoting legacy blockers as invalid can be useful; the durable prompt now
    // rewrites exact dangerous strings, so keep this exemption narrow.
    pattern == "read-only" && line.contains("not sandbox gates")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_fixture() -> String {
        r#"
# CODEX GPT-5.5 FIRST-RUN - ADVANCED AGENTIC VIBE CODING HARNESS v3 FULL ACCESS NO SANDBOX
## FULL-ACCESS NO-SANDBOX VARIANT
codex --dangerously-bypass-approvals-and-sandbox
- full local filesystem access is the baseline;
- network access is enabled;
- sandboxing is skipped;
- `approval_policy = "never"`;
- `sandbox_mode = "danger-full-access"`;
- `default_permissions = ":danger-full-access"`;
- do not call `request_permissions`;
- Phase 11 implementation must execute under full access/no sandbox and record unsupported features as evidence, not stop the run.
flag again --dangerously-bypass-approvals-and-sandbox
flag third --dangerously-bypass-approvals-and-sandbox
PHASE 11
- codex exec --json with full-access verification prompt
Run real full-access drill
Under the full-access no-sandbox controller
GitHub mutation uses github guard and full-access context
Do not claim complete if any command failed; record fail/unsupported/gap and continue repair under full access.
"#
        .to_string()
    }

    #[test]
    fn accepts_minimal_full_access_prompt_fixture() {
        let report = review_full_access_prompt_text(&valid_fixture());
        assert_prompt_review_ok(&report).unwrap();
        assert_eq!(report.launch_flag_count, 3);
    }

    #[test]
    fn rejects_read_only_failure_loop_phrases() {
        let text = format!("{}\nStay read-only\n", valid_fixture());
        let report = review_full_access_prompt_text(&text);
        assert!(!report.ok);
        assert!(
            report
                .forbidden_findings
                .iter()
                .any(|finding| finding.category == "read_only_mode")
        );
    }

    #[test]
    fn rejects_approval_and_stop_gates() {
        let text = format!(
            "{}\nIf subagent execution is not available, stop\nNo mutation before Phase 0 completes\nreview/approval\n",
            valid_fixture()
        );
        let report = review_full_access_prompt_text(&text);
        assert!(!report.ok);
        for category in ["subagent_stop", "mutation_ban", "approval_blocker"] {
            assert!(
                report
                    .forbidden_findings
                    .iter()
                    .any(|finding| finding.category == category),
                "missing category {category}: {:?}",
                report.forbidden_findings
            );
        }
    }

    #[test]
    fn rejects_missing_phase11_truth_mode() {
        let text = valid_fixture().replace(
            "Do not claim complete if any command failed; record fail/unsupported/gap and continue repair under full access.",
            "Do not claim complete if any command failed.",
        );
        let report = review_full_access_prompt_text(&text);
        assert!(!report.ok);
        assert!(
            report
                .missing_phase11_anchors
                .contains(&"failure_truth_mode".to_string())
        );
    }
}
