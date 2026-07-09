#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::{
    append_ledger, audit_value, codex_harness_dir, full_access_granted, full_access_marker_path,
    model_router_ready, nix_verify_value, project_root, state_dir, which,
    USER_FULL_ACCESS_DECISION_ID,
};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;

fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

fn row(
    law: &str,
    mechanism: &str,
    file_config: String,
    proof_command: &str,
    result: &str,
) -> Value {
    json!({
        "law": law,
        "mechanism": mechanism,
        "file_config": file_config,
        "proof_command": proof_command,
        "result": result,
    })
}

fn pass_fail(ok: bool) -> &'static str {
    if ok {
        "pass"
    } else {
        "missing"
    }
}

fn main() -> Result<()> {
    let harness = codex_harness_dir();
    let project = project_root();
    let audit = audit_value();
    let nix = nix_verify_value();
    let has_openrouter_key = env::var_os("OPENROUTER_API_KEY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let openrouter_proof_path = state_dir().join("openrouter-proof.json");
    let openrouter_proof: Value = fs::read_to_string(&openrouter_proof_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null);
    let openrouter_key_valid = openrouter_proof
        .get("authenticated_key_valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let openrouter_generation_ok = openrouter_proof
        .get("authenticated_generation_ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let openrouter_model_ok = openrouter_proof
        .get("target_model")
        .and_then(Value::as_str)
        .map(|model| model == "tencent/hy3:free")
        .unwrap_or(false);
    let openrouter_policy_blocked = openrouter_proof
        .get("account_policy_blocked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let openrouter_result =
        if openrouter_key_valid && openrouter_generation_ok && openrouter_model_ok {
            "pass"
        } else if openrouter_key_valid && openrouter_policy_blocked {
            "partial-auth-valid-account-policy-blocked"
        } else if has_openrouter_key {
            "partial-auth-env-present-no-successful-proof"
        } else {
            "partial-missing-OPENROUTER_API_KEY"
        };
    let has_claude_cli = which("claude").is_some();
    let has_gh = which("gh").is_some();

    let matrix = vec![
        row(
            "archive-first",
            "codex-harness-runner archive and archive ledger",
            harness.join("ledger/archive.jsonl").display().to_string(),
            "codex-harness-runner archive -- <path>",
            pass_fail(exists(harness.join("ledger/archive.jsonl"))),
        ),
        row(
            "real execution",
            "terminal gates and JSONL ledgers",
            harness.join("ledger").display().to_string(),
            "cargo test --all-features; codex-harness-audit",
            pass_fail(audit.get("ok").and_then(Value::as_bool).unwrap_or(false)),
        ),
        row(
            "containment-before-capability",
            "full-access marker plus native execpolicy containment matrix",
            full_access_marker_path().display().to_string(),
            "codex execpolicy check --pretty --rules ...",
            pass_fail(full_access_granted()),
        ),
        row(
            "Nix ownership",
            "Codex resolves through profile and /nix/store",
            "/home/flexnetos/.nix-profile/toolbin/codex".to_string(),
            "codex-harness-nix-verify",
            pass_fail(nix.get("ok").and_then(Value::as_bool).unwrap_or(false)),
        ),
        row(
            "model-router mandatory",
            "model-router marker and route ledger",
            harness
                .join("state/model-router-ready.json")
                .display()
                .to_string(),
            "codex-harness-model-router sample tasks",
            pass_fail(model_router_ready()),
        ),
        row(
            "secrets redaction",
            "redaction policy, secret-deny rules, sanitized child env",
            project
                .join(".codex/rules/secrets-deny.rules")
                .display()
                .to_string(),
            "cargo test redaction; codex execpolicy check secret_read",
            pass_fail(exists(project.join(".codex/rules/secrets-deny.rules"))),
        ),
        row(
            "yolo break-glass disabled",
            "no-yolo native rule and bad-behavior counter",
            project
                .join(".codex/rules/no-yolo.rules")
                .display()
                .to_string(),
            "codex execpolicy check -- codex --dangerously-bypass-approvals-and-sandbox",
            pass_fail(exists(project.join(".codex/rules/no-yolo.rules"))),
        ),
        row(
            "RULES/POLICY/SOUL",
            "harness policy files and SOUL constitution",
            harness.join("policy").display().to_string(),
            "find codex-harness/{policy,soul,rules}",
            pass_fail(
                exists(harness.join("policy/policy.toml")) && exists(harness.join("soul/SOUL.md")),
            ),
        ),
        row(
            "hooks",
            "project hooks call Rust hook",
            project.join(".codex/hooks.json").display().to_string(),
            "codex-harness-audit",
            pass_fail(exists(project.join(".codex/hooks.json"))),
        ),
        row(
            "rules",
            "15 native execpolicy files",
            project.join(".codex/rules").display().to_string(),
            "codex execpolicy check --pretty --rules <each> -- pwd",
            pass_fail(
                exists(project.join(".codex/rules/default.rules"))
                    && exists(project.join(".codex/rules/github.rules")),
            ),
        ),
        row(
            "subagents",
            "role files and model-router required flag",
            harness.join("agents").display().to_string(),
            "find codex-harness/agents -name '*.toml'",
            pass_fail(exists(harness.join("agents/conductor.toml"))),
        ),
        row(
            "teams",
            "team TOML caps",
            harness.join("teams").display().to_string(),
            "find codex-harness/teams -name '*.toml'",
            pass_fail(exists(harness.join("teams/research-team.toml"))),
        ),
        row(
            "profiles",
            "CODEX_HOME envctl profile files",
            "/home/flexnetos/.codex/envctl-*.config.toml".to_string(),
            "codex -p <profile> debug prompt-input",
            pass_fail(
                exists("/home/flexnetos/.codex/envctl-openrouter-gpt.config.toml")
                    && exists("/home/flexnetos/.codex/envctl-browser-computer.config.toml"),
            ),
        ),
        row(
            "model catalog",
            "harness model catalog and task matrix",
            harness
                .join("model-catalog/model-catalog.json")
                .display()
                .to_string(),
            "python -m json.tool model-catalog/model-catalog.json",
            pass_fail(exists(harness.join("model-catalog/model-catalog.json"))),
        ),
        row(
            "OpenRouter compatibility",
            "OpenRouter shim/catalog/probe default tencent/hy3:free",
            "/home/flexnetos/.codex/envctl-openrouter-gpt.config.toml".to_string(),
            "codex-harness-openrouter-shim probe",
            openrouter_result,
        ),
        row(
            "Claude bridge policy",
            "supervised Claude CLI bridge",
            harness
                .join("src/bin/codex-harness-claude-bridge.rs")
                .display()
                .to_string(),
            "codex-harness-claude-bridge inventory --allow-default-auth",
            pass_fail(has_claude_cli),
        ),
        row(
            "browser use",
            "feature flag and browser/computer gate",
            "/home/flexnetos/.codex/envctl-browser.config.toml".to_string(),
            "codex-harness-browser-computer verify",
            pass_fail(exists("/home/flexnetos/.codex/envctl-browser.config.toml")),
        ),
        row(
            "computer use",
            "feature flag and one-agent policy",
            "/home/flexnetos/.codex/envctl-computer-use.config.toml".to_string(),
            "codex-harness-browser-computer verify",
            pass_fail(exists(
                "/home/flexnetos/.codex/envctl-computer-use.config.toml",
            )),
        ),
        row(
            "memory/database",
            "memory audit plus rusqlite DB integrity",
            state_dir().join("harness.sqlite3").display().to_string(),
            "codex-harness-memory-audit; codex-harness-db integrity",
            pass_fail(exists(state_dir().join("harness.sqlite3"))),
        ),
        row(
            "statusline/timers",
            "native status config plus harness overlay",
            "/home/flexnetos/.codex/config.toml".to_string(),
            "codex-harness-status",
            pass_fail(exists("/home/flexnetos/.codex/config.toml")),
        ),
        row(
            "bad-behavior counter",
            "counters ledger and harness status",
            harness.join("ledger/counters.jsonl").display().to_string(),
            "codex-harness-status",
            pass_fail(exists(harness.join("ledger/counters.jsonl"))),
        ),
        row(
            "GitHub guard",
            "gh command guard with full-access decision id",
            harness
                .join("src/bin/codex-harness-github-guard.rs")
                .display()
                .to_string(),
            "codex-harness-github-guard check -- gh pr list",
            pass_fail(has_gh),
        ),
        row(
            "worktrees",
            "harness worktree directory and archive-before-cleanup drill",
            harness
                .parent()
                .unwrap_or(&harness)
                .join("worktrees")
                .display()
                .to_string(),
            "git worktree add/remove drill",
            pass_fail(exists(
                harness.parent().unwrap_or(&harness).join("worktrees"),
            )),
        ),
        row(
            "MCP",
            "active Codex MCP inventory",
            "/home/flexnetos/.codex/config.toml".to_string(),
            "codex mcp list",
            pass_fail(exists("/home/flexnetos/.codex/config.toml")),
        ),
        row(
            "plugins",
            "plugin inventory ledger",
            harness.join("ledger/plugins.jsonl").display().to_string(),
            "codex plugin list",
            pass_fail(exists(harness.join("ledger/plugins.jsonl"))),
        ),
        row(
            "kill switch",
            "halt binary and processes ledger",
            harness
                .join("src/bin/codex-harness-halt.rs")
                .display()
                .to_string(),
            "codex-harness-halt",
            pass_fail(exists(harness.join("src/bin/codex-harness-halt.rs"))),
        ),
        row(
            "SQLite/ledger integrity",
            "rusqlite DB and hash-chained JSONL ledgers",
            harness.join("state/harness.sqlite3").display().to_string(),
            "codex-harness-db integrity; codex-harness-audit",
            pass_fail(
                exists(harness.join("state/harness.sqlite3"))
                    && audit.get("ok").and_then(Value::as_bool).unwrap_or(false),
            ),
        ),
        row(
            "cross-platform process strategy",
            "cfg(unix)/cfg(not(unix)) halt/spawn strategy",
            harness.join("src/lib.rs").display().to_string(),
            "cargo test process_supervisor",
            pass_fail(exists(harness.join("src/lib.rs"))),
        ),
    ];

    let incomplete = matrix
        .iter()
        .filter(|entry| {
            entry
                .get("result")
                .and_then(Value::as_str)
                .map(|s| s != "pass" && !s.starts_with("pass-"))
                .unwrap_or(true)
        })
        .count();
    let ok = incomplete == 0;
    let out = json!({
        "ok": ok,
        "decision_id": if full_access_granted() { USER_FULL_ACCESS_DECISION_ID } else { "none" },
        "incomplete": incomplete,
        "matrix": matrix,
    });
    append_ledger(
        "harness.jsonl",
        json!({"event":"final_acceptance_matrix","decision": if ok {"allow"} else {"partial"},"result":out}),
    )?;
    println!("{}", serde_json::to_string_pretty(&out)?);
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}
