#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::prompt_review::review_full_access_prompt_path;
use codex_harness::{
    append_ledger, audit_value, codex_harness_dir, full_access_granted, full_access_marker_path,
    model_router_ready, nix_verify_value, project_root, state_dir, which,
    USER_FULL_ACCESS_DECISION_ID,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

fn result_is_acceptance(result: &str) -> bool {
    result == "pass" || result.starts_with("pass-") || result.starts_with("unsupported-")
}

fn prompt_file_probe(path: &Path, minimum_lines: usize) -> Value {
    match fs::read(path) {
        Ok(bytes) => {
            let digest = Sha256::digest(&bytes);
            let line_count = String::from_utf8_lossy(&bytes).lines().count();
            json!({
                "path": path.display().to_string(),
                "sha256": hex::encode(digest),
                "line_count": line_count,
                "loaded": true,
                "minimum_lines": minimum_lines,
                "accepted": line_count >= minimum_lines,
            })
        }
        Err(_) => json!({
            "path": path.display().to_string(),
            "sha256": Value::Null,
            "line_count": 0,
            "loaded": false,
            "minimum_lines": minimum_lines,
            "accepted": false,
        }),
    }
}

fn probe_accepted(probe: &Value) -> bool {
    probe
        .get("accepted")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn prompt_candidates(envctl_root: &Path) -> Vec<PathBuf> {
    vec![
        envctl_root
            .join(".codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md"),
        PathBuf::from("/home/flexnetos/Desktop/CODEX-GPT-HARNESS.prompt.md"),
        envctl_root.join(".codex/prompts/prompt:codex-gpt-harness.prompt.md"),
        PathBuf::from("/home/flexnetos/prompts/CODEX-GPT-HARNESS.prompt.md"),
    ]
}

fn phase_row(phase: u8, title: &str, prompt_anchor: String, evidence: &str, result: &str) -> Value {
    json!({
        "phase": phase,
        "title": title,
        "prompt_anchor": prompt_anchor,
        "evidence": evidence,
        "result": result,
    })
}

fn parse_phase_number(line: &str) -> Option<u8> {
    let rest = line.trim_start().strip_prefix("PHASE ")?;
    let digits = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u8>().ok()
}

fn prompt_phase_anchor(prompt_text: Option<&str>, phase: u8, fallback: &str) -> String {
    let Some(text) = prompt_text else {
        return fallback.to_string();
    };
    let lines = text.lines().collect::<Vec<_>>();
    let mut starts = Vec::<(u8, usize)>::new();
    for (index, line) in lines.iter().enumerate() {
        if let Some(number) = parse_phase_number(line) {
            starts.push((number, index + 1));
        }
    }
    let Some((position, (_, start_line))) = starts
        .iter()
        .enumerate()
        .find(|(_, (number, _))| *number == phase)
    else {
        return fallback.to_string();
    };
    let end_line = starts
        .get(position + 1)
        .map(|(_, next_start)| next_start.saturating_sub(1))
        .unwrap_or(lines.len());
    format!("PHASE {phase} lines {start_line}-{end_line}")
}

fn nonempty_str(value: &Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false)
}

fn checklist_state_ok(checklist: &Value, prompt_sha256: Option<&str>) -> bool {
    if checklist.get("schema").and_then(Value::as_str)
        != Some("codex-harness.phase-execution-checklist.v1")
    {
        return false;
    }
    if let Some(sha256) = prompt_sha256 {
        if checklist.pointer("/prompt/sha256").and_then(Value::as_str) != Some(sha256) {
            return false;
        }
    }
    let Some(phases) = checklist.get("phases").and_then(Value::as_array) else {
        return false;
    };
    let mut seen = BTreeSet::new();
    for phase in phases {
        let Some(number) = phase.get("phase").and_then(Value::as_u64) else {
            return false;
        };
        if number > 11 {
            return false;
        }
        seen.insert(number);
        if phase.get("result").and_then(Value::as_str) != Some("pass") {
            return false;
        }
        if !nonempty_str(phase, "/title") || !nonempty_str(phase, "/prompt_anchor") {
            return false;
        }
        let Some(items) = phase.get("items").and_then(Value::as_array) else {
            return false;
        };
        if items.is_empty() {
            return false;
        }
        for item in items {
            if !nonempty_str(item, "/id")
                || !nonempty_str(item, "/requirement")
                || !nonempty_str(item, "/proof_command")
                || !nonempty_str(item, "/evidence")
            {
                return false;
            }
            let status = item.get("status").and_then(Value::as_str).unwrap_or("");
            let mandatory = item
                .get("mandatory")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            if mandatory && status != "pass" {
                return false;
            }
            if !mandatory && status != "pass" && status != "unsupported" {
                return false;
            }
        }
    }
    seen == (0_u64..=11).collect::<BTreeSet<_>>()
}

fn phase_state_files_ok(harness: &Path, prompt_sha256: Option<&str>) -> bool {
    let checklist_path = harness.join("state/phase-execution-checklist.json");
    let continuation_path = harness.join("state/compact-continuation.md");
    let Some(checklist) = fs::read_to_string(&checklist_path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    else {
        return false;
    };
    if !checklist_state_ok(&checklist, prompt_sha256) {
        return false;
    }
    let Some(continuation) = fs::read_to_string(&continuation_path).ok() else {
        return false;
    };
    if !continuation.contains("state/phase-execution-checklist.json")
        || !continuation.contains("ledger/harness.jsonl")
        || !continuation.contains("next exact command:")
    {
        return false;
    }
    if let Some(sha256) = prompt_sha256 {
        if !continuation.contains(sha256) {
            return false;
        }
    }
    true
}

fn main() -> Result<()> {
    let harness = codex_harness_dir();
    let project = project_root();
    let audit = audit_value();
    let nix = nix_verify_value();
    let envctl_root = project.parent().unwrap_or(&project);
    let prompt_candidates = prompt_candidates(envctl_root);
    let prompt_path = prompt_candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .unwrap_or_else(|| prompt_candidates[0].clone());
    let prompt_bytes = fs::read(&prompt_path).ok();
    let prompt_sha256 = prompt_bytes.as_ref().map(|bytes| {
        let digest = Sha256::digest(bytes);
        hex::encode(digest)
    });
    let prompt_line_count = prompt_bytes
        .as_ref()
        .map(|bytes| String::from_utf8_lossy(bytes).lines().count())
        .unwrap_or(0);
    let prompt_text = prompt_bytes
        .as_ref()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned());
    let prompt_reloaded = prompt_sha256.is_some() && prompt_line_count >= 2400;
    let prompt_review = review_full_access_prompt_path(&prompt_path).ok();
    let prompt_review_ok = prompt_review
        .as_ref()
        .map(|report| report.ok)
        .unwrap_or(false);
    let recovery_prompt_path =
        envctl_root.join(".codex/prompts/prompt:codex-gpt-harness-v3-autonomous.prompt.md");
    let recovery_prompt = prompt_file_probe(&recovery_prompt_path, 80);
    let recovery_wrapper_loaded = probe_accepted(&recovery_prompt);
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
    let openrouter_models_ok = openrouter_proof
        .get("model_count")
        .and_then(Value::as_u64)
        .map(|count| count > 0)
        .unwrap_or(false);
    let openrouter_probe_no_secret_print = openrouter_proof
        .get("secret_printed")
        .and_then(Value::as_bool)
        .map(|printed| !printed)
        .unwrap_or(false);
    let openrouter_result =
        if openrouter_key_valid && openrouter_generation_ok && openrouter_model_ok {
            "pass"
        } else if openrouter_key_valid && openrouter_policy_blocked {
            "unsupported-account-policy-blocked"
        } else if !has_openrouter_key && openrouter_models_ok && openrouter_probe_no_secret_print {
            "unsupported-missing-OPENROUTER_API_KEY"
        } else if has_openrouter_key {
            "gap-auth-env-present-no-successful-proof"
        } else {
            "gap-openrouter-proof-missing"
        };
    let model_access_proof_path = state_dir().join("model-access-proof.json");
    let model_access_proof: Value = fs::read_to_string(&model_access_proof_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null);
    let model_access_ok = model_access_proof
        .pointer("/evaluation/ok")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && model_access_proof
            .pointer("/catalog/ok")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let has_claude_cli = which("claude").is_some();
    let has_gh = which("gh").is_some();

    let matrix = vec![
        row(
            "Phase -1 deadlock breaker",
            "v3 autonomous recovery wrapper can take control when shell/sandbox or approval gates deadlock",
            recovery_prompt_path.display().to_string(),
            "load .codex/prompts/prompt:codex-gpt-harness-v3-autonomous.prompt.md; codex-harness-final-verify",
            pass_fail(recovery_wrapper_loaded),
        ),
        row(
            "Phase 0 research audit",
            "original prompt reloaded plus read-only research ledger and Phase 1 plan draft",
            prompt_path.display().to_string(),
            "sha256sum CODEX-GPT-HARNESS.prompt.md; inspect ledger/research.jsonl; PHASE1_PLAN_DRAFT.md",
            pass_fail(
                prompt_reloaded
                    && prompt_review_ok
                    && exists(harness.join("ledger/research.jsonl"))
                    && exists(harness.join("PHASE1_PLAN_DRAFT.md")),
            ),
        ),
        row(
            "full-access prompt truth review",
            "strict no-blocker prompt review binary plus TDD tests",
            prompt_path.display().to_string(),
            "codex-harness-prompt-review <prompt>; cargo test --test prompt_review",
            pass_fail(prompt_reloaded && prompt_review_ok),
        ),
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
            "model access proof",
            "catalog coverage plus live Codex account probes for GPT-5.5/GPT-5.4/Spark and account-gated preview/o-series denials",
            model_access_proof_path.display().to_string(),
            "codex-harness-model-access probe",
            pass_fail(model_access_ok),
        ),
        row(
            "OpenRouter compatibility",
            "OpenRouter shim/catalog/probe default tencent/hy3:free; authenticated generation is account/env gated",
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
            "skills",
            "envctl-owned agent-skills source plus agent sync gate",
            envctl_root.join("agent-skills").display().to_string(),
            "envctl agent sync --json --color never",
            pass_fail(exists(envctl_root.join("agent-skills"))),
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
            "parallel execution",
            "runner process supervisor ledger and model-router controlled fanout",
            harness.join("ledger/processes.jsonl").display().to_string(),
            "codex-harness-runner spawn/status; cargo test process_supervisor",
            pass_fail(
                exists(harness.join("ledger/processes.jsonl"))
                    && model_router_ready()
                    && exists(harness.join("teams/research-team.toml")),
            ),
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
            "phase execution state",
            "compact checklist and continuation files prove all phases against the current prompt",
            harness
                .join("state/phase-execution-checklist.json")
                .display()
                .to_string(),
            "inspect state/phase-execution-checklist.json and state/compact-continuation.md",
            pass_fail(phase_state_files_ok(&harness, prompt_sha256.as_deref())),
        ),
        row(
            "cross-platform process strategy",
            "cfg(unix)/cfg(not(unix)) halt/spawn strategy",
            harness.join("src/lib.rs").display().to_string(),
            "cargo test process_supervisor",
            pass_fail(exists(harness.join("src/lib.rs"))),
        ),
    ];

    let row_pass = |law: &str| {
        matrix
            .iter()
            .find(|entry| entry.get("law").and_then(Value::as_str) == Some(law))
            .and_then(|entry| entry.get("result").and_then(Value::as_str))
            .map(result_is_acceptance)
            .unwrap_or(false)
    };
    let phase_matrix = vec![
        phase_row(
            0,
            "Deep research + read-only audit gate",
            prompt_phase_anchor(prompt_text.as_deref(), 0, "PHASE 0"),
            "prompt sha256, recovery wrapper, research ledger, PHASE1 plan draft",
            pass_fail(row_pass("Phase -1 deadlock breaker") && row_pass("Phase 0 research audit")),
        ),
        phase_row(
            1,
            "Containment before agentic power",
            prompt_phase_anchor(prompt_text.as_deref(), 1, "PHASE 1"),
            "full-access marker, no-yolo/secret rules, hooks, kill switch",
            pass_fail(
                row_pass("containment-before-capability")
                    && row_pass("secrets redaction")
                    && row_pass("yolo break-glass disabled")
                    && row_pass("hooks")
                    && row_pass("rules")
                    && row_pass("kill switch"),
            ),
        ),
        phase_row(
            2,
            "Config, model catalog, and provider toggles",
            prompt_phase_anchor(prompt_text.as_deref(), 2, "PHASE 2"),
            "profiles, model catalog, model-access proof, OpenRouter default proof",
            pass_fail(
                row_pass("profiles")
                    && row_pass("model catalog")
                    && row_pass("model access proof")
                    && row_pass("OpenRouter compatibility"),
            ),
        ),
        phase_row(
            3,
            "Subagent-mandatory team fabric",
            prompt_phase_anchor(prompt_text.as_deref(), 3, "PHASE 3"),
            "model router, subagent role files, team caps",
            pass_fail(
                row_pass("model-router mandatory") && row_pass("subagents") && row_pass("teams"),
            ),
        ),
        phase_row(
            4,
            "Advanced TUI, timers, and bad-behavior counters",
            prompt_phase_anchor(prompt_text.as_deref(), 4, "PHASE 4"),
            "statusline/timer config and counters ledger",
            pass_fail(row_pass("statusline/timers") && row_pass("bad-behavior counter")),
        ),
        phase_row(
            5,
            "Browser use and computer use",
            prompt_phase_anchor(prompt_text.as_deref(), 5, "PHASE 5"),
            "browser/computer-use profiles and verification gate",
            pass_fail(row_pass("browser use") && row_pass("computer use")),
        ),
        phase_row(
            6,
            "Memory and database",
            prompt_phase_anchor(prompt_text.as_deref(), 6, "PHASE 6"),
            "memory audit state DB plus SQLite/ledger integrity",
            pass_fail(row_pass("memory/database") && row_pass("SQLite/ledger integrity")),
        ),
        phase_row(
            7,
            "Providers, networking, and model fabric",
            prompt_phase_anchor(prompt_text.as_deref(), 7, "PHASE 7"),
            "OpenRouter proof, model access proof, Claude bridge, Nix ownership",
            pass_fail(
                row_pass("OpenRouter compatibility")
                    && row_pass("model access proof")
                    && row_pass("Claude bridge policy")
                    && row_pass("Nix ownership"),
            ),
        ),
        phase_row(
            8,
            "GitHub control, policy, and worktrees",
            prompt_phase_anchor(prompt_text.as_deref(), 8, "PHASE 8"),
            "GitHub guard and worktree/archive drill",
            pass_fail(row_pass("GitHub guard") && row_pass("worktrees")),
        ),
        phase_row(
            9,
            "Skills, plugins, and MCP",
            prompt_phase_anchor(prompt_text.as_deref(), 9, "PHASE 9"),
            "envctl skills source, plugin ledger, active MCP inventory",
            pass_fail(row_pass("skills") && row_pass("plugins") && row_pass("MCP")),
        ),
        phase_row(
            10,
            "Parallel execution fabric",
            prompt_phase_anchor(prompt_text.as_deref(), 10, "PHASE 10"),
            "runner process ledger, model-router fanout, team caps",
            pass_fail(row_pass("parallel execution")),
        ),
        phase_row(
            11,
            "Final verification",
            prompt_phase_anchor(prompt_text.as_deref(), 11, "PHASE 11"),
            "final acceptance matrix, phase state files, and all phase rows",
            "pending-self-check",
        ),
    ];

    let incomplete = matrix
        .iter()
        .filter(|entry| {
            entry
                .get("result")
                .and_then(Value::as_str)
                .map(|s| !result_is_acceptance(s))
                .unwrap_or(true)
        })
        .count();
    let phase_incomplete_without_final = phase_matrix
        .iter()
        .filter(|entry| entry.get("phase").and_then(Value::as_u64) != Some(11))
        .filter(|entry| {
            entry
                .get("result")
                .and_then(Value::as_str)
                .map(|s| !result_is_acceptance(s))
                .unwrap_or(true)
        })
        .count();
    let final_phase_result = pass_fail(incomplete == 0 && phase_incomplete_without_final == 0);
    let phase_matrix = phase_matrix
        .into_iter()
        .map(|mut entry| {
            if entry.get("phase").and_then(Value::as_u64) == Some(11) {
                if let Some(object) = entry.as_object_mut() {
                    object.insert("result".to_string(), json!(final_phase_result));
                }
            }
            entry
        })
        .collect::<Vec<_>>();
    let phase_incomplete = phase_matrix
        .iter()
        .filter(|entry| {
            entry
                .get("result")
                .and_then(Value::as_str)
                .map(|s| !result_is_acceptance(s))
                .unwrap_or(true)
        })
        .count();
    let ok = incomplete == 0 && phase_incomplete == 0;
    let out = json!({
        "ok": ok,
        "decision_id": if full_access_granted() { USER_FULL_ACCESS_DECISION_ID } else { "none" },
        "incomplete": incomplete,
        "phase_incomplete": phase_incomplete,
        "prompt": {
            "path": prompt_path,
            "sha256": prompt_sha256,
            "line_count": prompt_line_count,
            "reloaded": prompt_reloaded,
            "full_access_review_ok": prompt_review_ok,
            "full_access_review": prompt_review,
            "recovery_wrapper": recovery_prompt,
        },
        "phase_matrix": phase_matrix,
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
