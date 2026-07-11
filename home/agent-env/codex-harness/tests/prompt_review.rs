#![forbid(unsafe_code)]

use codex_harness::prompt_review::{
    assert_prompt_review_ok, review_full_access_prompt_path, review_full_access_prompt_text,
};
use std::path::PathBuf;

fn envctl_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .nth(3)
        .expect("harness crate path has envctl ancestor")
        .to_path_buf()
}

fn original_prompt_path() -> PathBuf {
    envctl_root().join(".codex/prompts/prompt:codex-gpt-harness.prompt.md")
}

fn full_access_prompt_path() -> PathBuf {
    envctl_root()
        .join(".codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md")
}

#[test]
fn both_repo_prompt_entrypoints_are_identical_and_have_no_blockers() {
    let original = std::fs::read(original_prompt_path()).unwrap();
    let full_access = std::fs::read(full_access_prompt_path()).unwrap();
    assert_eq!(
        original, full_access,
        "the original prompt entrypoint must remain byte-identical to the full-access prompt"
    );
    let full_access_text = String::from_utf8(full_access.clone()).unwrap();
    for required_skill_path in [
        "references/source-prompt.md",
        "references/ownership-map.md",
        "references/runbook-cli-contract.md",
        "references/coverage-map.md",
        "references/bunx-and-github-ssh.md",
        "references/github-execution-policy.md",
        "references/github-org-and-ccboard.md",
        "references/yazelix-cli-plugin-policy.md",
        "scripts/check-bun-command-policy.py",
        "scripts/check-yazelix-contract.py",
        "scripts/validate.sh",
    ] {
        assert!(
            full_access_text.contains(required_skill_path),
            "prompt target shape omits {required_skill_path}"
        );
    }
    assert!(
        !full_access_text.contains("`rtk git ...`"),
        "prompt must route every repository Git operation through RTK/Meta"
    );
    assert!(
        !full_access_text.contains("/bin/rtk git status"),
        "prompt must not retain a direct RTK Git probe"
    );
    assert!(
        !full_access_text.contains("`rtk meta exec -- git"),
        "unlisted Git operations must identify the Meta repo scope"
    );
    assert!(
        !full_access_text.contains("`meta exec -- git"),
        "unwrapped Meta Git passthrough is forbidden"
    );
    assert!(
        full_access_text.contains("rtk meta exec --include <repo> -- git <command>"),
        "prompt must retain the scoped RTK/Meta fallback"
    );

    for prompt in [original_prompt_path(), full_access_prompt_path()] {
        let report = review_full_access_prompt_path(&prompt).unwrap();
        assert_prompt_review_ok(&report)
            .unwrap_or_else(|error| panic!("{}: {error}", prompt.display()));
        assert!(
            report.line_count > 2_400,
            "line_count={}",
            report.line_count
        );
        assert!(report.launch_flag_count >= 3);
    }
}

#[test]
fn wide_fan_blockers_fail_fast() {
    let required = r#"
FULL-ACCESS NO-SANDBOX VARIANT
codex --dangerously-bypass-approvals-and-sandbox
--dangerously-bypass-approvals-and-sandbox
--dangerously-bypass-approvals-and-sandbox
approval_policy = "never"
sandbox_mode = "danger-full-access"
default_permissions = ":danger-full-access"
full local filesystem access is the baseline
network access is enabled
sandboxing is skipped
do not call `request_permissions`
Phase 11 implementation must execute under full access/no sandbox
PHASE 11
codex exec --json with full-access verification prompt
Run real full-access drill
Under the full-access no-sandbox controller
GitHub mutation uses github guard and full-access context
record fail/unsupported/gap and continue repair under full access
"#;
    let blockers = [
        "read-only",
        "workspace-write",
        "sandbox_mode = \"read-only\"",
        "sandbox_mode = \"workspace-write\"",
        "Stay in a limited mode",
        "Do not edit files",
        "Do not create files",
        "No mutation before Phase 0 completes",
        "Do not proceed to Phase",
        "If subagent execution is not available, stop",
        "approval required",
        "review/approval",
        "unless approved",
        "No repo writes",
        "network disabled",
        "request_permissions_tool = true",
        "exec_permission_approvals = true",
        "budget ceiling: ask operator",
    ];
    for blocker in blockers {
        let report = review_full_access_prompt_text(&format!("{required}\n{blocker}\n"));
        assert!(!report.ok, "blocker was not caught: {blocker}");
        assert!(
            report
                .forbidden_findings
                .iter()
                .any(|finding| finding.snippet.contains(blocker)),
            "no finding contained blocker {blocker}: {:?}",
            report.forbidden_findings
        );
    }
}
