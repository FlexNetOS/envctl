#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_PROJECT_ROOT: &str = "/home/flexnetos/lifeos/src/envctl/home";
pub const DEFAULT_HARNESS_ROOT: &str = "/home/flexnetos/lifeos/src/envctl/home/agent-env";

pub const LEDGER_NAMES: &[&str] = &[
    "harness.jsonl",
    "processes.jsonl",
    "archive.jsonl",
    "budget.jsonl",
    "decisions.jsonl",
    "research.jsonl",
    "rules.jsonl",
    "policy.jsonl",
    "soul.jsonl",
    "subagents.jsonl",
    "counters.jsonl",
    "model_router.jsonl",
    "model-routing.jsonl",
    "network.jsonl",
    "browser-computer.jsonl",
    "plugins.jsonl",
    "mcp.jsonl",
    "bad-behavior.jsonl",
    "index.jsonl",
    "github.jsonl",
    "memory.jsonl",
];

pub const USER_FULL_ACCESS_DECISION_ID: &str = "USER-FULL-ACCESS-20260709";

pub mod prompt_review;

const REDACTED_INDEX_TABLE: redb::TableDefinition<&str, &str> =
    redb::TableDefinition::new("redacted_entries");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionKind {
    Allow,
    Prompt,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub decision: DecisionKind,
    pub reason: String,
    pub redacted_preview: String,
    pub violation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub job_id: String,
    pub pid: u32,
    pub pgid: Option<i64>,
    pub cwd: String,
    pub argv: Vec<String>,
    pub started_utc: String,
    pub status: String,
    pub log_path: String,
}

pub fn utc_now() -> String {
    let out = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output();
    if let Ok(out) = out {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    "1970-01-01T00:00:00Z".to_string()
}

pub fn monotonic_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

pub fn harness_root() -> PathBuf {
    let raw = env::var_os("CODEX_HARNESS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_HARNESS_ROOT));
    if raw.file_name() == Some(OsStr::new("codex-harness")) {
        return raw.parent().map(Path::to_path_buf).unwrap_or(raw);
    }
    raw
}

pub fn codex_harness_dir() -> PathBuf {
    harness_root().join("codex-harness")
}

pub fn project_root() -> PathBuf {
    env::var_os("CODEX_HARNESS_PROJECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_PROJECT_ROOT))
}

pub fn ledger_dir() -> PathBuf {
    codex_harness_dir().join("ledger")
}

pub fn state_dir() -> PathBuf {
    codex_harness_dir().join("state")
}

pub fn full_access_marker_path() -> PathBuf {
    state_dir().join("full-access-grant.json")
}

pub fn full_access_granted() -> bool {
    matches!(
        env::var("CODEX_HARNESS_FULL_ACCESS").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || full_access_marker_path().exists()
}

pub fn grant_full_access(reason: &str) -> Result<Value> {
    fs::create_dir_all(state_dir())?;
    let value = json!({
        "ok": true,
        "decision_id": USER_FULL_ACCESS_DECISION_ID,
        "ts_utc": utc_now(),
        "reason": redact(reason),
        "scope": [
            "danger-full-access",
            "browser-computer-use",
            "claude-bridge",
            "openrouter",
            "github-full-access"
        ],
        "secret_policy": "do not print or ledger secret values"
    });
    fs::write(
        full_access_marker_path(),
        serde_json::to_vec_pretty(&value)?,
    )?;
    append_ledger(
        "decisions.jsonl",
        json!({"event":"full_access_grant","decision":"allow","decision_id":USER_FULL_ACCESS_DECISION_ID,"reason":redact(reason)}),
    )?;
    Ok(value)
}

pub fn archive_encode_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "%2F")
        .replace('\\', "%5C")
}

pub fn sha256_path(path: &Path) -> Result<String> {
    let meta = fs::symlink_metadata(path)?;
    let mut hasher = Sha256::new();
    if meta.file_type().is_symlink() {
        hasher.update(b"symlink:");
        hasher.update(fs::read_link(path)?.to_string_lossy().as_bytes());
    } else if meta.is_file() {
        let mut file = fs::File::open(path)?;
        let mut buf = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buf)?;
            if read == 0 {
                break;
            }
            hasher.update(&buf[..read]);
        }
    } else if meta.is_dir() {
        hasher.update(b"dir:");
        for entry in sorted_dir_entries(path)? {
            let entry_path = entry?;
            let rel = entry_path.strip_prefix(path).unwrap_or(&entry_path);
            hasher.update(rel.to_string_lossy().as_bytes());
            hasher.update(b"\0");
            hasher.update(sha256_path(&entry_path)?.as_bytes());
            hasher.update(b"\0");
        }
    } else {
        hasher.update(b"other");
    }
    Ok(hex::encode(hasher.finalize()))
}

fn sorted_dir_entries(root: &Path) -> Result<Vec<Result<PathBuf>>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if fs::symlink_metadata(&path)?.is_dir() {
            for child in sorted_dir_entries(&path)? {
                out.push(child);
            }
        } else {
            out.push(Ok(path));
        }
    }
    out.sort_by(|a, b| {
        let a = a.as_ref().map(|p| p.to_string_lossy()).unwrap_or_default();
        let b = b.as_ref().map(|p| p.to_string_lossy()).unwrap_or_default();
        a.cmp(&b)
    });
    Ok(out)
}

fn copy_path_preserving(source: &Path, dest: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if meta.file_type().is_symlink() {
        let target = fs::read_link(source)?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, dest)?;
        #[cfg(not(unix))]
        {
            let _ = target;
            return Err(anyhow!(
                "symlink archive is not implemented on this platform"
            ));
        }
    } else if meta.is_dir() {
        fs::create_dir_all(dest)?;
        fs::set_permissions(dest, meta.permissions())?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_path_preserving(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else if meta.is_file() {
        fs::copy(source, dest)?;
        fs::set_permissions(dest, meta.permissions())?;
    } else {
        return Err(anyhow!(
            "unsupported archive file type: {}",
            source.display()
        ));
    }
    Ok(())
}

pub fn archive_path(source: &Path, reason: &str) -> Result<Value> {
    let absolute = if source.is_absolute() {
        source.to_path_buf()
    } else {
        env::current_dir()?.join(source)
    };
    if !absolute.exists()
        && !fs::symlink_metadata(&absolute)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    {
        return Err(anyhow!(
            "archive source does not exist: {}",
            absolute.display()
        ));
    }
    let ts = utc_now();
    let dest = harness_root()
        .join("archive")
        .join(&ts)
        .join(archive_encode_path(&absolute));
    let meta = fs::symlink_metadata(&absolute)?;
    let symlink_target = if meta.file_type().is_symlink() {
        fs::read_link(&absolute)?.display().to_string()
    } else {
        String::new()
    };
    let sha_before = sha256_path(&absolute)?;
    copy_path_preserving(&absolute, &dest)?;
    let sha_archive = sha256_path(&dest)?;
    #[cfg(unix)]
    let (mode, owner, group) = {
        use std::os::unix::fs::MetadataExt;
        (
            format!("{:o}", meta.mode() & 0o7777),
            meta.uid().to_string(),
            meta.gid().to_string(),
        )
    };
    #[cfg(not(unix))]
    let (mode, owner, group) = (
        if meta.permissions().readonly() {
            "readonly"
        } else {
            "writable"
        }
        .to_string(),
        String::new(),
        String::new(),
    );
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    let event = json!({
        "event":"archive_before_modify",
        "decision":"archive",
        "reason":reason,
        "source":absolute.display().to_string(),
        "archive_path":dest.display().to_string(),
        "sha256_before":sha_before,
        "sha256_archive":sha_archive,
        "mode":mode,
        "owner":owner,
        "group":group,
        "mtime":mtime,
        "symlink_target":symlink_target,
        "session_id": Value::Null,
        "parent_id": Value::Null
    });
    append_ledger("archive.jsonl", event.clone())?;
    Ok(event)
}

pub fn restore_archive(archive: &Path, dest: &Path) -> Result<Value> {
    if !archive.exists()
        && !fs::symlink_metadata(archive)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    {
        return Err(anyhow!(
            "archive path does not exist: {}",
            archive.display()
        ));
    }
    if dest.exists() {
        return Err(anyhow!(
            "restore destination already exists: {}",
            dest.display()
        ));
    }
    copy_path_preserving(archive, dest)?;
    let event = json!({
        "event":"archive_restore",
        "decision":"restore",
        "reason":"restore archive copy to requested destination",
        "archive_path":archive.display().to_string(),
        "restore_path":dest.display().to_string(),
        "sha256_archive":sha256_path(archive)?,
        "sha256_restore":sha256_path(dest)?,
    });
    append_ledger("archive.jsonl", event.clone())?;
    Ok(event)
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

pub fn redact(input: &str) -> String {
    let mut out = input.to_string();
    let lower = input.to_ascii_lowercase();
    let sensitive_markers = [
        "auth.json",
        ".env",
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "id_token",
        "bearer ",
        "authorization:",
        "private key",
        "id_rsa",
        "id_ed25519",
        "secret",
        "password",
    ];
    if sensitive_markers.iter().any(|m| lower.contains(m)) {
        out = "[REDACTED:SENSITIVE-COMMAND]".to_string();
    }
    for key in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENROUTER_API_KEY",
        "GITHUB_TOKEN",
        "GH_TOKEN",
        "ANTHROPIC_API_KEY",
        "CLAUDE_API_KEY",
    ] {
        out = out.replace(key, "[REDACTED_ENV_KEY]");
    }
    out
}

pub fn command_preview(argv: &[String]) -> String {
    redact(&argv.join(" "))
}

pub fn sanitized_argv(argv: &[String]) -> Vec<String> {
    argv.iter().map(|arg| redact(arg)).collect()
}

fn remove_secret_env(command: &mut Command) {
    for key in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENROUTER_API_KEY",
        "ANTHROPIC_API_KEY",
        "CLAUDE_API_KEY",
        "GITHUB_TOKEN",
        "GH_TOKEN",
        "GITLAB_TOKEN",
        "BITBUCKET_TOKEN",
    ] {
        command.env_remove(key);
    }
}

pub fn policy_decision(argv: &[String]) -> PolicyDecision {
    let preview = command_preview(argv);
    let command = argv.join(" ");
    let lower = command.to_ascii_lowercase();
    let first = argv.first().map(|s| basename(s)).unwrap_or_default();

    let decision = |kind: DecisionKind, reason: &str, violation: Option<&str>| PolicyDecision {
        decision: kind,
        reason: reason.to_string(),
        redacted_preview: preview.clone(),
        violation: violation.map(str::to_string),
    };
    let deny =
        |reason: &str, violation: &str| decision(DecisionKind::Deny, reason, Some(violation));
    let allow = |reason: &str| decision(DecisionKind::Allow, reason, None);
    let prompt =
        |reason: &str, violation: Option<&str>| decision(DecisionKind::Prompt, reason, violation);

    if argv.is_empty() {
        return deny("empty command", "empty_command");
    }

    if first.starts_with("codex-harness-") {
        if known_harness_command(&first) {
            return allow("known Codex harness command is managed by the harness");
        }
        return deny(
            "unknown codex-harness-* command is not trusted by prefix alone",
            "unknown_harness_command",
        );
    }

    if preview.contains("[REDACTED:SENSITIVE-COMMAND]") || secret_path_violation(&lower) {
        return deny("secret-bearing command or path is forbidden", "secret_read");
    }

    if direct_state_mutation_violation(&lower) {
        return deny(
            "direct ledger/archive/index mutation is forbidden; use sanctioned harness flow",
            "direct_ledger_or_index_mutation",
        );
    }

    if danger_or_bypass_requested(&lower) {
        if full_access_granted() {
            return allow("danger/full-access request allowed by explicit user full-access grant");
        }
        if contains_decision_id(argv)
            || lower.contains("decision_id")
            || lower.contains("decision-id")
        {
            return prompt(
                "danger/yolo/full-access request has a decision id but still requires explicit human approval",
                Some("danger_requires_human_approval"),
            );
        }
        return deny(
            "yolo/danger/full-access/bypass request is forbidden without decision id",
            "danger_without_decision_id",
        );
    }

    if ["rm", "unlink", "rmdir"].contains(&first.as_str()) {
        return deny(
            "delete command forbidden; use archive flow",
            "delete_without_archive",
        );
    }

    if uncontrolled_background_requested(&first, &lower) {
        return deny(
            "background escape must go through codex-harness-runner",
            "uncontrolled_background_job",
        );
    }

    if nested_model_provider(&first, &lower) {
        return deny(
            "nested Codex/Claude/model provider launch must go through codex-harness-runner and model-router",
            "nested_model_provider_without_runner",
        );
    }

    if browser_or_computer_use(&first, &lower) {
        if full_access_granted() {
            return allow("browser/computer-use allowed by explicit user full-access grant");
        }
        if browser_profile_approved(argv) {
            return prompt(
                "browser/computer-use has approved profile marker but still requires human approval",
                Some("browser_requires_human_approval"),
            );
        }
        return deny(
            "browser/computer-use is forbidden without approved profile",
            "browser_without_profile",
        );
    }

    if first == "git" {
        if git_mutation(argv) {
            return deny(
                "GitHub/git mutation requires codex-harness-github-guard",
                "github_mutation_without_guard",
            );
        }
        if git_network(argv) {
            return deny(
                "unmanaged git network command is forbidden by containment",
                "unmanaged_network",
            );
        }
        return allow("read-only git command allowed");
    }

    if first == "gh" {
        if full_access_granted() {
            return allow("GitHub full-access command allowed by explicit user grant");
        }
        if gh_mutation(argv) {
            return deny(
                "GitHub mutation requires codex-harness-github-guard",
                "github_mutation_without_guard",
            );
        }
        return prompt(
            "GitHub read command uses network; run only as explicit foreground proof",
            Some("network_read"),
        );
    }

    if unmanaged_network_requested(&first, &lower) {
        return deny(
            "unmanaged network command is forbidden by containment",
            "unmanaged_network",
        );
    }

    if first == "envctl" {
        if argv
            .iter()
            .any(|a| a == "--apply" || a == "--build" || a == "--purge")
        {
            return prompt(
                "envctl mutating/apply command requires explicit review outside harness policy auto-allow",
                Some("mutating_apply_requires_review"),
            );
        }
        return allow("envctl preview/read-only command allowed");
    }

    let read_only = [
        "pwd", "true", "false", "date", "whoami", "ls", "cat", "sed", "rg", "grep", "find", "fd",
        "nix", "jq", "sleep", "wc", "stat", "head", "tail", "cargo", "rustfmt",
    ];
    if read_only.contains(&first.as_str()) {
        if first == "cargo" && argv.iter().any(|a| a == "install" || a == "publish") {
            return deny(
                "cargo install/publish forbidden in containment",
                "unmanaged_install",
            );
        }
        if first == "cargo" && argv.iter().any(|a| a == "build") {
            return prompt(
                "cargo build requires explicit foreground proof",
                Some("build_requires_review"),
            );
        }
        return allow("read-only or verification command allowlisted");
    }

    if ["nix-build", "nix", "nix-shell", "nix-store"].contains(&first.as_str()) {
        return prompt(
            "Nix command requires explicit foreground proof",
            Some("build_requires_review"),
        );
    }

    prompt(
        "command not explicitly allowlisted",
        Some("unknown_command"),
    )
}

fn known_harness_command(first: &str) -> bool {
    matches!(
        first,
        "codex-harness-audit"
            | "codex-harness-cargo-clippy"
            | "codex-harness-cargo-fmt"
            | "codex-harness-github-guard"
            | "codex-harness-halt"
            | "codex-harness-hook"
            | "codex-harness-index"
            | "codex-harness-jsonl"
            | "codex-harness-memory-audit"
            | "codex-harness-memory-disable-plan"
            | "codex-harness-memory-export-redacted"
            | "codex-harness-model-router"
            | "codex-harness-nix-verify"
            | "codex-harness-openrouter-shim"
            | "codex-harness-policy"
            | "codex-harness-runner"
            | "codex-harness-claude-bridge"
            | "codex-harness-browser-computer"
            | "codex-harness-db"
            | "codex-harness-final-verify"
            | "codex-harness-status"
    )
}

fn has_arg(argv: &[String], names: &[&str]) -> bool {
    argv.iter().any(|arg| names.iter().any(|name| arg == name))
}

fn contains_decision_id(argv: &[String]) -> bool {
    for (idx, arg) in argv.iter().enumerate() {
        if arg == "--decision-id" {
            return argv
                .get(idx + 1)
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false);
        }
        if arg.starts_with("--decision-id=")
            || arg.starts_with("decision_id=")
            || arg.starts_with("decision-id=")
        {
            return arg
                .split_once('=')
                .map(|(_, v)| !v.trim().is_empty())
                .unwrap_or(false);
        }
    }
    false
}

fn secret_path_violation(lower: &str) -> bool {
    let denied = [
        "/home/flexnetos/.codex/auth.json",
        "~/.codex/auth.json",
        "/.codex/auth.json",
        "/home/flexnetos/.gnupg",
        "~/.gnupg",
        "/.gnupg",
        "/home/flexnetos/.ssh",
        "~/.ssh",
        "/.ssh",
        "vault_hub",
    ];
    if denied.iter().any(|needle| lower.contains(needle)) {
        return true;
    }
    lower.contains(".env")
        || lower.contains("private-key")
        || lower.contains("private_key")
        || lower.contains("id_rsa")
        || lower.contains("id_ed25519")
}

fn direct_state_mutation_violation(lower: &str) -> bool {
    lower.contains("codex-harness/ledger")
        || lower.contains("/ledger/")
        || lower.contains("agent-env/archive")
        || lower.contains("/archive/")
        || lower.contains("codex-harness/state/index")
        || lower.contains("/state/index/")
        || lower.contains(".redb")
}

fn danger_or_bypass_requested(lower: &str) -> bool {
    lower.contains("danger-full-access")
        || lower.contains("full-access")
        || lower.contains("dangerously-bypass")
        || lower.contains("--yolo")
        || lower.contains(" yolo")
        || lower.contains("bypass")
        || lower.contains("ignore-rules")
}

fn uncontrolled_background_requested(first: &str, lower: &str) -> bool {
    ["tmux", "screen", "nohup", "disown", "setsid"].contains(&first)
        || lower.contains(" &")
        || lower.contains("start-job")
        || lower.contains("start-process")
        || lower.contains("systemd-run")
}

fn nested_model_provider(first: &str, lower: &str) -> bool {
    [
        "codex",
        "claude",
        "ollama",
        "lms",
        "lmstudio",
        "openai",
        "anthropic",
        "openrouter",
    ]
    .contains(&first)
        || lower.contains("openrouter")
        || lower.contains("claude-code")
        || lower.contains("computer-use-preview")
}

fn browser_or_computer_use(first: &str, lower: &str) -> bool {
    [
        "playwright",
        "chromium",
        "google-chrome",
        "google-chrome-stable",
        "firefox",
        "computer-use",
    ]
    .contains(&first)
        || lower.contains("computer use")
        || lower.contains("computer-use")
        || lower.contains("browser")
}

fn browser_profile_approved(argv: &[String]) -> bool {
    argv.windows(2).any(|pair| {
        matches!(pair[0].as_str(), "--profile" | "--browser-profile")
            && pair[1] == "envctl-harness-browser-approved"
    }) || argv.iter().any(|arg| {
        arg == "--approved-browser-profile=envctl-harness-browser-approved"
            || arg == "--profile=envctl-harness-browser-approved"
            || arg == "--browser-profile=envctl-harness-browser-approved"
    })
}

fn git_mutation(argv: &[String]) -> bool {
    has_arg(argv, &["push", "clean", "tag"])
        || (has_arg(argv, &["reset"]) && has_arg(argv, &["--hard"]))
        || (has_arg(argv, &["branch"]) && has_arg(argv, &["-D", "--delete", "-d"]))
        || has_arg(argv, &["merge", "rebase", "cherry-pick", "commit"])
}

fn git_network(argv: &[String]) -> bool {
    has_arg(argv, &["fetch", "pull", "clone", "submodule"])
}

fn gh_mutation(argv: &[String]) -> bool {
    let readonly = matches!(
        (
            argv.get(1).map(String::as_str),
            argv.get(2).map(String::as_str)
        ),
        (Some("pr"), Some("list" | "view" | "checks" | "status"))
            | (Some("run"), Some("view" | "list"))
            | (Some("issue"), Some("list" | "view"))
            | (Some("repo"), Some("view"))
            | (Some("auth"), Some("status"))
    );
    !readonly
}

fn unmanaged_network_requested(first: &str, lower: &str) -> bool {
    [
        "curl", "wget", "ssh", "scp", "sftp", "rsync", "nc", "ncat", "telnet", "ftp",
    ]
    .contains(&first)
        || lower.contains("curl ")
        || lower.contains("wget ")
        || lower.contains("ssh ")
        || lower.contains("https://")
        || lower.contains("http://")
}

fn basename(s: &str) -> String {
    Path::new(s)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(s)
        .to_string()
}

pub fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(items) => {
            let inner = items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        Value::Object(map) => {
            let mut items = map.iter().collect::<Vec<_>>();
            items.sort_by(|a, b| a.0.cmp(b.0));
            let inner = items
                .into_iter()
                .map(|(k, v)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap(),
                        canonical_json(v)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{inner}}}")
        }
    }
}

pub fn append_ledger(name: &str, mut event: Value) -> Result<String> {
    fs::create_dir_all(ledger_dir())?;
    let path = ledger_dir().join(name);
    let prev = last_line_hash(&path)?.unwrap_or_else(|| "GENESIS".to_string());
    let seq = count_lines(&path)? + 1;
    {
        let obj = event
            .as_object_mut()
            .ok_or_else(|| anyhow!("ledger event must be object"))?;
        obj.insert("ts_utc".into(), json!(utc_now()));
        obj.insert("seq".into(), json!(seq));
        obj.insert(
            "cwd".into(),
            json!(env::current_dir()?.display().to_string()),
        );
        obj.insert("prev_hash".into(), json!(prev));
    }
    let body = canonical_json(&event);
    let line_hash = sha256_bytes(body.as_bytes());
    event
        .as_object_mut()
        .ok_or_else(|| anyhow!("ledger event must be object"))?
        .insert("line_hash".into(), json!(line_hash.clone()));
    let line = canonical_json(&event);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{line}")?;
    Ok(line_hash)
}

pub fn count_lines(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    let file = fs::File::open(path)?;
    Ok(io::BufReader::new(file).lines().count() as u64)
}

pub fn last_line_hash(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let file = fs::File::open(path)?;
    let mut last = None;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line)?;
        last = v
            .get("line_hash")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    Ok(last)
}

pub fn verify_ledger(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let file = fs::File::open(path)?;
    let mut prev = "GENESIS".to_string();
    let mut count = 0usize;
    for (idx, line) in io::BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let mut v: Value = serde_json::from_str(&line)
            .with_context(|| format!("{} line {}", path.display(), idx + 1))?;
        let obj = v
            .as_object_mut()
            .ok_or_else(|| anyhow!("ledger line not object"))?;
        let got_prev = obj.get("prev_hash").and_then(Value::as_str).unwrap_or("");
        if got_prev != prev {
            return Err(anyhow!(
                "{} line {} prev_hash mismatch",
                path.display(),
                idx + 1
            ));
        }
        let got_hash = obj
            .remove("line_hash")
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| anyhow!("{} line {} missing line_hash", path.display(), idx + 1))?;
        let body = canonical_json(&v);
        let want = sha256_bytes(body.as_bytes());
        if got_hash != want {
            return Err(anyhow!(
                "{} line {} line_hash mismatch",
                path.display(),
                idx + 1
            ));
        }
        prev = got_hash;
        count += 1;
    }
    Ok(count)
}

pub fn read_stdin_string() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

pub fn hook_response(input: &Value) -> Result<Value> {
    let event = input
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("Unknown");
    if budget_ceiling_exceeded()
        && matches!(
            event,
            "PreToolUse" | "PermissionRequest" | "SubagentStart" | "UserPromptSubmit"
        )
    {
        record_bad_behavior("budget_ceiling", "budget ceiling marker present", "")?;
        append_ledger(
            "budget.jsonl",
            json!({"event":"budget_ceiling_block","decision":"deny","reason":"budget ceiling marker present"}),
        )?;
        return Ok(
            json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness budget ceiling exceeded"},"decision":"block","reason":"codex-harness budget ceiling exceeded","systemMessage":"codex-harness blocked because budget ceiling marker is present"}),
        );
    }
    if matches!(event, "SubagentStart") {
        let depth = input
            .get("depth")
            .or_else(|| input.pointer("/subagent/depth"))
            .and_then(Value::as_i64)
            .unwrap_or(1);
        if depth > 1 {
            record_bad_behavior("subagent_depth", "max_depth=1", &format!("depth={depth}"))?;
            append_ledger(
                "decisions.jsonl",
                json!({"event":"subagent_depth_deny","decision":"deny","reason":"max_depth=1","depth":depth}),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness max_depth=1 denies depth > 1"},"systemMessage":"codex-harness denied subagent depth > 1"}),
            );
        }
        if !model_router_ready() {
            record_bad_behavior(
                "subagent_without_model_router",
                "subagent spawn requires model-router first",
                &redact(&input.to_string()),
            )?;
            append_ledger(
                "decisions.jsonl",
                json!({"event":"subagent_model_router_deny","decision":"deny","reason":"model-router route marker missing"}),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness denied subagent spawn before model-router"},"systemMessage":"codex-harness denied subagent spawn before model-router"}),
            );
        }
    }
    let mut command_argv = Vec::<String>::new();
    if let Some(cmd) = input.pointer("/tool_input/command").and_then(Value::as_str) {
        command_argv = vec!["sh".into(), "-lc".into(), cmd.into()];
    }
    if let Some(cmd) = input.pointer("/tool_input/cmd").and_then(Value::as_str) {
        command_argv = vec!["sh".into(), "-lc".into(), cmd.into()];
    }
    if let Some(arr) = input.pointer("/tool_input/argv").and_then(Value::as_array) {
        command_argv = arr
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
    }
    let text = input.to_string();
    if matches!(event, "UserPromptSubmit") {
        let lower = text.to_ascii_lowercase();
        if !full_access_granted()
            && (lower.contains("disable hooks")
                || lower.contains("danger-full-access")
                || lower.contains("bypass")
                || lower.contains("leak secrets")
                || lower.contains("without archive"))
        {
            record_bad_behavior(
                "unsafe_user_prompt",
                "prompt requests bypass or secret unsafe action",
                &redact(&text),
            )?;
            append_ledger(
                "decisions.jsonl",
                json!({"event":"user_prompt_block","decision":"deny","reason":"prompt requests bypass or secret unsafe action","preview":redact(&text)}),
            )?;
            return Ok(
                json!({"decision":"block","reason":"codex-harness blocked bypass/secret/archive violation request"}),
            );
        }
    }
    if matches!(event, "PermissionRequest") {
        let lower = text.to_ascii_lowercase();
        if !full_access_granted()
            && (lower.contains("danger") || lower.contains("bypass") || lower.contains("auth.json"))
        {
            record_bad_behavior(
                "unsafe_permission_request",
                "danger/bypass/secret permission request",
                &redact(&text),
            )?;
            append_ledger(
                "decisions.jsonl",
                json!({"event":"permission_deny","decision":"deny","reason":"danger/bypass/secret permission request","preview":redact(&text)}),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness denied danger/bypass/secret permission request"},"systemMessage":"codex-harness denied unsafe permission request"}),
            );
        }
    }
    if matches!(event, "PreToolUse") && !command_argv.is_empty() {
        let decision = policy_decision(&command_argv);
        append_ledger(
            "decisions.jsonl",
            json!({"event":"pre_tool_use","decision":format!("{:?}", decision.decision),"reason":decision.reason,"preview":decision.redacted_preview}),
        )?;
        if decision.decision == DecisionKind::Deny {
            record_policy_violation(&decision)?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":decision.reason},"systemMessage":"codex-harness denied unsafe tool use"}),
            );
        }
    }
    if matches!(event, "PreToolUse") {
        let lower = text.to_ascii_lowercase();
        if lower.contains("codex-harness/ledger")
            || lower.contains("/ledger/")
            || lower.contains("agent-env/archive")
            || lower.contains("/archive/")
            || lower.contains("auth.json")
            || lower.contains(".ssh")
            || lower.contains(".env")
        {
            record_bad_behavior(
                "unsafe_path_access",
                "direct ledger/archive/index/secret path mutation or read",
                &redact(&text),
            )?;
            append_ledger(
                "decisions.jsonl",
                json!({"event":"pre_tool_use_path_deny","decision":"deny","reason":"direct ledger/archive/secret path mutation or read","preview":redact(&text)}),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness denied direct ledger/archive/secret path access"},"systemMessage":"codex-harness denied unsafe path access"}),
            );
        }
        let tool_name = input
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let path = input
            .pointer("/tool_input/path")
            .or_else(|| input.pointer("/tool_input/file_path"))
            .and_then(Value::as_str);
        if matches!(tool_name, "apply_patch" | "Edit" | "Write") {
            if let Some(path) = path {
                let target = Path::new(path);
                if target.exists() && !archive_record_exists(path)? {
                    record_bad_behavior(
                        "write_without_archive",
                        "existing target has no archive record",
                        path,
                    )?;
                    append_ledger(
                        "decisions.jsonl",
                        json!({"event":"write_without_archive_deny","decision":"deny","reason":"existing target has no archive record","target":path}),
                    )?;
                    return Ok(
                        json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness denied write/edit without archive record"},"systemMessage":"codex-harness denied write/edit without archive record"}),
                    );
                }
            }
        }
    }
    if matches!(event, "Stop") && unresolved_decision_exists()? {
        let marker = state_dir().join("stop-blocked-once");
        if !marker.exists() {
            fs::create_dir_all(state_dir())?;
            fs::write(&marker, utc_now())?;
            record_bad_behavior(
                "stop_unresolved_decision",
                "unresolved decision marker exists",
                "",
            )?;
            append_ledger(
                "decisions.jsonl",
                json!({"event":"stop_block_once","decision":"deny","reason":"unresolved decision marker exists"}),
            )?;
            return Ok(
                json!({"decision":"block","reason":"codex-harness unresolved decision marker blocks Stop once"}),
            );
        }
    }
    append_ledger(
        "harness.jsonl",
        json!({"event":"hook_allow","hook_event_name":event,"decision":"allow"}),
    )?;
    Ok(json!({}))
}

pub fn unresolved_decision_exists() -> Result<bool> {
    Ok(state_dir().join("unresolved-decision").exists())
}

pub fn budget_ceiling_exceeded() -> bool {
    state_dir().join("budget-ceiling-exceeded").exists()
}

pub fn archive_record_exists(source: &str) -> Result<bool> {
    let path = ledger_dir().join("archive.jsonl");
    if !path.exists() {
        return Ok(false);
    }
    let raw = PathBuf::from(source);
    let absolute = if raw.is_absolute() {
        raw
    } else {
        env::current_dir()?.join(raw)
    };
    let absolute_string = absolute.display().to_string();
    let canonical = fs::canonicalize(&absolute).ok();
    let canonical_string = canonical.as_ref().map(|p| p.display().to_string());
    let file = fs::File::open(path)?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line)?;
        let Some(record_source) = v.get("source").and_then(Value::as_str) else {
            continue;
        };
        if record_source == source || record_source == absolute_string {
            return Ok(true);
        }
        if canonical_string.as_deref() == Some(record_source) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn last_deny_summary() -> Result<String> {
    let path = ledger_dir().join("decisions.jsonl");
    if !path.exists() {
        return Ok("none".to_string());
    }
    let mut last = "none".to_string();
    for line in io::BufReader::new(fs::File::open(path)?).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line)?;
        let decision = v.get("decision").and_then(Value::as_str).unwrap_or("");
        if decision.eq_ignore_ascii_case("deny") || decision.eq_ignore_ascii_case("forbidden") {
            let reason = v.get("reason").and_then(Value::as_str).unwrap_or("deny");
            let preview = v.get("preview").and_then(Value::as_str).unwrap_or("");
            last = redact(&format!("{reason} {preview}"));
        }
    }
    Ok(last)
}

pub fn nix_verify_value() -> Value {
    let codex_path = which("codex");
    let realpath = codex_path.as_ref().and_then(|p| fs::canonicalize(p).ok());
    let path_entries = env::var_os("PATH")
        .map(|p| env::split_paths(&p).collect::<Vec<_>>())
        .unwrap_or_default();
    let shadows: Vec<String> = path_entries
        .iter()
        .filter_map(|entry| {
            let candidate = entry.join("codex");
            if candidate.exists() {
                Some(candidate.display().to_string())
            } else {
                None
            }
        })
        .collect();
    let profile_owned = codex_path
        .as_ref()
        .map(|p| {
            p.starts_with("/home/flexnetos/.nix-profile")
                || p.to_string_lossy().contains("/.nix-profile/")
        })
        .unwrap_or(false);
    let store_owned = realpath
        .as_ref()
        .map(|p| p.starts_with("/nix/store"))
        .unwrap_or(false);
    let first_shadow_ok = shadows
        .first()
        .map(|p| {
            p == "/home/flexnetos/.nix-profile/bin/codex"
                || p.starts_with("/home/flexnetos/.nix-profile/")
        })
        .unwrap_or(false);
    json!({
        "codex_path": codex_path.map(|p| p.display().to_string()),
        "realpath": realpath.map(|p| p.display().to_string()),
        "shadows": shadows,
        "profile_owned": profile_owned,
        "store_owned": store_owned,
        "first_shadow_ok": first_shadow_ok,
        "ok": profile_owned && store_owned && first_shadow_ok
    })
}

pub fn which(bin: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for entry in env::split_paths(&path) {
        let candidate = entry.join(bin);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn active_job_records() -> Result<Vec<JobRecord>> {
    let dir = state_dir().join("jobs");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut jobs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(OsStr::to_str) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(entry.path())?;
        let mut job: JobRecord = serde_json::from_str(&text)?;
        if job.status == "running" {
            if process_alive(job.pid) {
                jobs.push(job);
            } else {
                job.status = "stopped".into();
                write_job_record(&job)?;
            }
        }
    }
    Ok(jobs)
}

pub fn process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

pub fn write_job_record(job: &JobRecord) -> Result<()> {
    let dir = state_dir().join("jobs");
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join(format!("{}.json", job.job_id)),
        serde_json::to_vec_pretty(job)?,
    )?;
    Ok(())
}

pub fn spawn_supervised(cwd: &Path, argv: &[String]) -> Result<JobRecord> {
    let decision = policy_decision(argv);
    if decision.decision != DecisionKind::Allow {
        record_policy_violation(&decision)?;
        return Err(anyhow!("{:?}: {}", decision.decision, decision.reason));
    }
    spawn_supervised_unchecked(cwd, argv)
}

fn job_kind(argv: &[String]) -> &'static str {
    match argv
        .first()
        .map(|s| basename(s))
        .unwrap_or_default()
        .as_str()
    {
        "codex" => "codex",
        "claude" => "claude",
        "ollama" | "lms" | "lmstudio" => "local-model",
        _ => "background",
    }
}

fn enforce_job_caps(argv: &[String]) -> Result<()> {
    let active = active_job_records()?;
    if active.len() >= 6 {
        return Err(anyhow!("max concurrent harness jobs exceeded"));
    }
    let kind = job_kind(argv);
    let same_kind = active
        .iter()
        .filter(|job| job_kind(&job.argv) == kind)
        .count();
    match kind {
        "codex" if same_kind >= 3 => Err(anyhow!("max concurrent Codex child sessions exceeded")),
        "local-model" if same_kind >= 3 => Err(anyhow!("max concurrent local-model jobs exceeded")),
        "claude" if same_kind >= 1 => Err(anyhow!("max concurrent Claude child sessions exceeded")),
        _ => Ok(()),
    }
}

pub fn spawn_supervised_unchecked(cwd: &Path, argv: &[String]) -> Result<JobRecord> {
    enforce_job_caps(argv)?;
    let job_id = monotonic_id("job");
    let logs = state_dir().join("logs");
    fs::create_dir_all(&logs)?;
    let log_path = logs.join(format!("{job_id}.log"));
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let mut command;
    #[cfg(unix)]
    {
        command = Command::new("setsid");
        command.arg(&argv[0]).args(&argv[1..]);
    }
    #[cfg(not(unix))]
    {
        command = Command::new(&argv[0]);
        command.args(&argv[1..]);
    }
    let child = command
        .current_dir(cwd)
        .env_remove("OPENAI_API_KEY")
        .env_remove("CODEX_API_KEY")
        .env_remove("OPENROUTER_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("CLAUDE_API_KEY")
        .env_remove("GITHUB_TOKEN")
        .env_remove("GH_TOKEN")
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file))
        .spawn()
        .with_context(|| format!("spawn {:?}", argv))?;
    let pid = child.id();
    let job = JobRecord {
        job_id,
        pid,
        pgid: Some(pid as i64),
        cwd: cwd.display().to_string(),
        argv: sanitized_argv(argv),
        started_utc: utc_now(),
        status: "running".into(),
        log_path: log_path.display().to_string(),
    };
    write_job_record(&job)?;
    append_ledger(
        "processes.jsonl",
        json!({"event":"spawn","decision":"allow","job_id":job.job_id,"pid":pid,"kind":job_kind(argv),"command_hash":sha256_bytes(command_preview(argv).as_bytes()),"command_preview":command_preview(argv)}),
    )?;
    Ok(job)
}

pub fn spawn_codex_exec(cwd: &Path, profile: &str, prompt: &str) -> Result<JobRecord> {
    require_model_router_ready()?;
    let argv = vec![
        "codex".to_string(),
        "exec".to_string(),
        "--json".to_string(),
        "--profile".to_string(),
        profile.to_string(),
        prompt.to_string(),
    ];
    spawn_supervised_unchecked(cwd, &argv)
}

pub fn spawn_ollama_run(cwd: &Path, model: &str, prompt: &str) -> Result<JobRecord> {
    let argv = vec![
        "ollama".to_string(),
        "run".to_string(),
        model.to_string(),
        prompt.to_string(),
    ];
    spawn_supervised_unchecked(cwd, &argv)
}

pub fn run_codex_exec(cwd: &Path, profile: &str, prompt: &str) -> Result<i32> {
    require_model_router_ready()?;
    let preview = format!(
        "codex exec --json --profile {profile} [prompt_sha:{}]",
        sha256_bytes(prompt.as_bytes())
    );
    let mut command = Command::new("codex");
    command
        .args(["exec", "--json", "--profile", profile, prompt])
        .current_dir(cwd);
    remove_secret_env(&mut command);
    let output = command.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stdout.lines() {
        println!("{line}");
        if !line.trim().is_empty() {
            let event = serde_json::from_str::<Value>(line)
                .ok()
                .and_then(|v| v.get("type").or_else(|| v.get("event")).cloned())
                .unwrap_or_else(|| json!("unparsed"));
            append_ledger(
                "budget.jsonl",
                json!({"event":"codex_exec_jsonl","decision":"record","jsonl_event":event,"redacted":redact(line)}),
            )?;
        }
    }
    if !stderr.trim().is_empty() {
        eprintln!("{}", redact(&stderr));
    }
    let code = output.status.code().unwrap_or(1);
    append_ledger(
        "processes.jsonl",
        json!({"event":"codex_exec_complete","decision": if code == 0 {"allow"} else {"deny"},"exit_code":code,"command_preview":preview}),
    )?;
    Ok(code)
}

pub fn run_ollama(cwd: &Path, model: &str, prompt: &str) -> Result<i32> {
    let prompt_hash = sha256_bytes(prompt.as_bytes());
    let mut command = Command::new("ollama");
    command.args(["run", model, prompt]).current_dir(cwd);
    remove_secret_env(&mut command);
    let output = command.output()?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", redact(&String::from_utf8_lossy(&output.stderr)));
    let code = output.status.code().unwrap_or(1);
    append_ledger(
        "processes.jsonl",
        json!({"event":"ollama_run_complete","decision": if code == 0 {"allow"} else {"deny"},"exit_code":code,"model":model,"prompt_hash":prompt_hash}),
    )?;
    Ok(code)
}

pub fn halt_jobs(dry_run: bool) -> Result<Value> {
    let mut stopped = Vec::new();
    let mut survivors = Vec::new();
    for mut job in active_job_records()? {
        if dry_run {
            survivors.push(json!({"job_id":job.job_id,"pid":job.pid,"dry_run":true}));
            continue;
        }
        if !process_alive(job.pid) {
            job.status = "stopped".into();
            write_job_record(&job)?;
            stopped.push(json!({"job_id":job.job_id,"pid":job.pid,"already_exited":true}));
            continue;
        }
        #[cfg(unix)]
        let status = Command::new("kill")
            .arg("-TERM")
            .arg("--")
            .arg(format!("-{}", job.pgid.unwrap_or(job.pid as i64)))
            .status()
            .or_else(|_| {
                Command::new("kill")
                    .arg("-TERM")
                    .arg(job.pid.to_string())
                    .status()
            });
        #[cfg(not(unix))]
        let status = Command::new("taskkill")
            .args(["/PID", &job.pid.to_string(), "/T", "/F"])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            job.status = "stopped".into();
            write_job_record(&job)?;
            stopped.push(json!({"job_id":job.job_id,"pid":job.pid}));
        } else {
            survivors.push(json!({"job_id":job.job_id,"pid":job.pid}));
        }
    }
    append_ledger(
        "processes.jsonl",
        json!({"event":"halt","decision":"allow","dry_run":dry_run,"stopped":stopped,"survivors":survivors}),
    )?;
    Ok(json!({"stopped":stopped,"survivors":survivors}))
}

pub fn audit_value() -> Value {
    let mut ledgers = BTreeMap::new();
    let mut ok = true;
    for name in LEDGER_NAMES {
        let path = ledger_dir().join(name);
        match verify_ledger(&path) {
            Ok(lines) => {
                ledgers.insert(name.to_string(), json!({"ok":true,"lines":lines}));
            }
            Err(err) => {
                ok = false;
                ledgers.insert(
                    name.to_string(),
                    json!({"ok":false,"error":err.to_string()}),
                );
            }
        }
    }
    let project = project_root();
    let hooks = project.join(".codex/hooks.json");
    let config = project.join(".codex/config.toml");
    json!({
        "ok": ok,
        "harness_root": harness_root(),
        "project_root": project,
        "ledgers": ledgers,
        "project_hooks_exists": hooks.exists(),
        "project_config_exists": config.exists(),
        "nix": nix_verify_value()
    })
}

pub fn jsonl_parse_stdin() -> Result<Value> {
    let stdin = io::stdin();
    let mut parsed = 0usize;
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        let event = value
            .get("type")
            .or_else(|| value.get("event"))
            .cloned()
            .unwrap_or(json!("unknown"));
        append_ledger(
            "budget.jsonl",
            json!({"event":"codex_jsonl","decision":"record","jsonl_event":event,"redacted":redact(&line)}),
        )?;
        parsed += 1;
    }
    Ok(json!({"parsed":parsed}))
}

pub fn run_foreground(cwd: &Path, argv: &[String]) -> Result<i32> {
    let decision = policy_decision(argv);
    if decision.decision != DecisionKind::Allow {
        record_policy_violation(&decision)?;
        append_ledger(
            "processes.jsonl",
            json!({"event":"run_denied","decision":format!("{:?}", decision.decision),"reason":decision.reason,"command_preview":decision.redacted_preview}),
        )?;
        return Err(anyhow!("{:?}: {}", decision.decision, decision.reason));
    }
    let status = Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .status()?;
    let code = status.code().unwrap_or(1);
    append_ledger(
        "processes.jsonl",
        json!({"event":"run","decision":"allow","exit_code":code,"command_hash":sha256_bytes(command_preview(argv).as_bytes()),"command_preview":command_preview(argv)}),
    )?;
    Ok(code)
}

pub fn record_policy_violation(decision: &PolicyDecision) -> Result<()> {
    let kind = decision
        .violation
        .as_deref()
        .unwrap_or("policy_prompt_or_deny");
    record_bad_behavior(kind, &decision.reason, &decision.redacted_preview)
}

pub fn record_bad_behavior(kind: &str, reason: &str, preview: &str) -> Result<()> {
    append_ledger(
        "counters.jsonl",
        json!({
            "event":"bad_behavior_counter",
            "decision":"deny",
            "kind":kind,
            "reason":reason,
            "preview":redact(preview)
        }),
    )?;
    Ok(())
}

pub fn bad_behavior_counts() -> Result<BTreeMap<String, u64>> {
    let mut counts = BTreeMap::new();
    let path = ledger_dir().join("counters.jsonl");
    if !path.exists() {
        return Ok(counts);
    }
    let file = fs::File::open(path)?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        if value.get("event").and_then(Value::as_str) != Some("bad_behavior_counter") {
            continue;
        }
        let kind = value
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *counts.entry(kind).or_insert(0) += 1;
    }
    Ok(counts)
}

pub fn model_router_dir() -> PathBuf {
    state_dir().join("model-router")
}

pub fn model_router_marker() -> PathBuf {
    model_router_dir().join("last-route.json")
}

pub fn model_router_ready() -> bool {
    let marker = model_router_marker();
    marker.exists()
        && fs::read_to_string(marker)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| v.get("ok").and_then(Value::as_bool))
            .unwrap_or(false)
}

pub fn require_model_router_ready() -> Result<()> {
    if model_router_ready() {
        Ok(())
    } else {
        record_bad_behavior(
            "model_router_required",
            "model-router must approve route before model/subagent spawn",
            "",
        )?;
        Err(anyhow!(
            "model-router route marker missing; run codex-harness-model-router first"
        ))
    }
}

pub fn model_route_for_task(task: &str) -> Value {
    let lower = task.to_ascii_lowercase();
    let full = full_access_granted();
    let (class, profile, model, provider, reason) = if lower.contains("gpt-5.6")
        || lower.contains("sol")
        || lower.contains("terra")
        || lower.contains("luna")
    {
        (
            "model-access-preview",
            "envctl-gpt56-sol",
            "gpt-5.6-sol",
            "openai",
            "GPT-5.6 Sol/Terra/Luna are preview-gated; route only after codex-harness-model-access proves account access",
        )
    } else if lower.contains("spark") {
        (
            "model-access-spark",
            "envctl-spark",
            "gpt-5.3-codex-spark",
            "openai",
            "Spark route is explicitly tracked and must be proved by codex-harness-model-access",
        )
    } else if lower.contains("o3") || lower.contains("o4-mini") {
        (
            "model-access-reasoning",
            "envctl-o3",
            "o3",
            "openai",
            "o-series routing is account-gated for Codex ChatGPT sessions; require live model-access proof before use",
        )
    } else if lower.contains("model catalog") || lower.contains("model access") {
        (
            "model-access-audit",
            "envctl-gpt55-standard",
            "gpt-5.5",
            "openai",
            "model catalog/access audits use primary GPT-5.5 plus codex-harness-model-access probes",
        )
    } else if lower.contains("openrouter") {
        (
            "provider-openrouter",
            "envctl-openrouter-gpt",
            "tencent/hy3:free",
            "openrouter",
            "OpenRouter route explicitly enabled by user grant; default model set by operator and proof depends on OPENROUTER_API_KEY",
        )
    } else if lower.contains("claude") {
        (
            "provider-claude-bridge",
            "envctl-claude-bridge",
            "claude-sonnet-5",
            "claude-bridge",
            "Claude direct use is routed through supervised external claude CLI bridge",
        )
    } else if lower.contains("browser") || lower.contains("computer") || lower.contains("gui") {
        (
            "browser-computer",
            "envctl-browser-computer",
            "gpt-5.5",
            "openai",
            "Browser/Computer Use route enabled by user grant and gated through browser-computer policy",
        )
    } else if lower.contains("github") || lower.contains("gh ") {
        (
            "github-full-access",
            "envctl-github-full-access",
            "gpt-5.5",
            "openai",
            "GitHub route uses codex-harness-github-guard with full-access decision id",
        )
    } else if lower.contains("security")
        || lower.contains("containment")
        || lower.contains("policy")
    {
        (
            "containment",
            "envctl-harness",
            "active-codex-default",
            "codex-profile-frontdoor",
            "security/containment work stays on active Codex profile",
        )
    } else if lower.contains("index") || lower.contains("memory") || lower.contains("ledger") {
        (
            "local-proof",
            "envctl-harness",
            "active-codex-default",
            "codex-profile-frontdoor",
            "ledger/index/memory work uses local Rust proof; JSONL remains canonical",
        )
    } else {
        (
            "implementation",
            "envctl-harness",
            "active-codex-default",
            "codex-profile-frontdoor",
            "default implementation route",
        )
    };
    json!({
        "task": task,
        "class": class,
        "provider": provider,
        "profile": profile,
        "model": model,
        "approved_capability_expansion": full,
        "full_access_decision_id": if full {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
        "openrouter_enabled": full,
        "claude_bridge_enabled": full,
        "browser_computer_enabled": full,
        "github_full_access_enabled": full,
        "reason": reason,
    })
}

pub fn route_model_tasks(tasks: &[String]) -> Result<Value> {
    if tasks.is_empty() {
        return Err(anyhow!("model-router requires at least one task"));
    }
    let routes = tasks
        .iter()
        .map(|task| model_route_for_task(task))
        .collect::<Vec<_>>();
    let value = json!({
        "ok": true,
        "route_id": monotonic_id("route"),
        "ts_utc": utc_now(),
        "requires_runner": true,
        "routes": routes,
        "containment": {
            "subagent_spawn_requires_this_marker": true,
            "provider_expansion_allowed": full_access_granted(),
            "full_access_decision_id": if full_access_granted() {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
            "openrouter_shim": if full_access_granted() {"enabled"} else {"disabled_pending_approval"},
            "claude_bridge": if full_access_granted() {"enabled"} else {"disabled_pending_approval"},
            "browser_computer_use": if full_access_granted() {"enabled"} else {"disabled_pending_approved_profile"},
            "github_full_access": if full_access_granted() {"enabled"} else {"disabled_pending_decision"}
        }
    });
    fs::create_dir_all(model_router_dir())?;
    fs::write(model_router_marker(), serde_json::to_vec_pretty(&value)?)?;
    append_ledger(
        "model_router.jsonl",
        json!({"event":"route","decision":"allow","route":value}),
    )?;
    Ok(value)
}

pub fn sample_model_tasks() -> Vec<String> {
    vec![
        "containment policy tests".to_string(),
        "redacted index integrity".to_string(),
        "memory audit disable plan".to_string(),
        "OpenRouter Responses proof".to_string(),
        "model catalog and model access proof".to_string(),
        "GPT-5.6 Sol Terra Luna access proof".to_string(),
        "Spark and o3 access proof".to_string(),
        "Claude bridge proof".to_string(),
        "Browser Computer Use proof".to_string(),
        "GitHub full access proof".to_string(),
    ]
}

fn index_dir() -> PathBuf {
    state_dir().join("index")
}

fn redacted_index_path() -> PathBuf {
    index_dir().join("redacted.redb")
}

pub fn index_integrity_check() -> Result<Value> {
    fs::create_dir_all(index_dir())?;
    let index_path = redacted_index_path();
    if index_path.exists() {
        archive_path(
            &index_path,
            "codex-harness-index overwrite redacted derived index",
        )?;
    }
    let db = redb::Database::create(&index_path)?;
    let mut indexed = 0usize;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(REDACTED_INDEX_TABLE)?;
        for ledger in LEDGER_NAMES {
            let path = ledger_dir().join(ledger);
            verify_ledger(&path)?;
            if !path.exists() {
                continue;
            }
            let file = fs::File::open(&path)?;
            for line in io::BufReader::new(file).lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let value: Value = serde_json::from_str(&line)?;
                let seq = value.get("seq").and_then(Value::as_u64).unwrap_or(0);
                let key = format!("{ledger}:{seq}");
                let redacted_line = redact(&line);
                let entry = json!({
                    "ledger": ledger,
                    "seq": seq,
                    "event": value.get("event").cloned().unwrap_or(Value::Null),
                    "decision": value.get("decision").cloned().unwrap_or(Value::Null),
                    "reason": value.get("reason").and_then(Value::as_str).map(redact),
                    "redacted_json": redacted_line,
                    "source_hash": sha256_bytes(line.as_bytes()),
                });
                table.insert(key.as_str(), canonical_json(&entry).as_str())?;
                indexed += 1;
            }
        }
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(REDACTED_INDEX_TABLE)?;
    let mut read_back = 0usize;
    for item in table.iter()? {
        let (_key, value) = item?;
        let text = value.value();
        if secret_path_violation(&text.to_ascii_lowercase()) {
            return Err(anyhow!("redacted index contains denied secret path marker"));
        }
        read_back += 1;
    }
    let out = json!({
        "ok": indexed == read_back,
        "backend": "redb",
        "canonical_source": "jsonl_ledgers",
        "sqlite_default": false,
        "index_path": index_path.display().to_string(),
        "indexed_entries": indexed,
        "read_back_entries": read_back,
    });
    append_ledger(
        "index.jsonl",
        json!({"event":"integrity_check","decision":"allow","result":out}),
    )?;
    Ok(out)
}

pub fn github_guard_check(
    argv: &[String],
    decision_id: Option<&str>,
    execute: bool,
) -> Result<Value> {
    if argv.is_empty() {
        return Err(anyhow!("github-guard requires a command after --"));
    }
    let first = basename(&argv[0]);
    if first != "gh" && first != "git" {
        return Err(anyhow!("github-guard only accepts gh or git commands"));
    }
    let mutation = (first == "gh" && gh_mutation(argv)) || (first == "git" && git_mutation(argv));
    if mutation
        && decision_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_none()
        && !full_access_granted()
    {
        record_bad_behavior(
            "github_mutation_without_guard_decision",
            "GitHub mutation requested without decision id",
            &command_preview(argv),
        )?;
        return Ok(json!({
            "ok": false,
            "decision": "deny",
            "reason": "GitHub mutation requires --decision-id",
            "mutation": mutation,
            "redacted_preview": command_preview(argv),
        }));
    }
    let mut result = json!({
        "ok": true,
        "decision": if mutation {"guarded"} else {"read_only"},
        "mutation": mutation,
        "executed": false,
        "decision_id": decision_id.or_else(|| full_access_granted().then_some(USER_FULL_ACCESS_DECISION_ID)),
        "redacted_preview": command_preview(argv),
    });
    if execute {
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]);
        remove_secret_env(&mut command);
        let status = command.status()?;
        result["executed"] = json!(true);
        result["exit_code"] = json!(status.code().unwrap_or(1));
    }
    append_ledger(
        "github.jsonl",
        json!({"event":"github_guard","decision":result.get("decision").cloned().unwrap_or(Value::Null),"result":result}),
    )?;
    Ok(result)
}

fn command_output_text(mut command: Command) -> Result<(i32, String, String)> {
    remove_secret_env(&mut command);
    let output = command.output()?;
    Ok((
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

pub fn openrouter_probe_value(model: Option<&str>, prompt: Option<&str>) -> Result<Value> {
    let model = model.unwrap_or("tencent/hy3:free");
    let prompt = prompt.unwrap_or("Return the single word pong.");
    let has_key = env::var_os("OPENROUTER_API_KEY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    let mut models_cmd = Command::new("curl");
    models_cmd.args(["-sS", "https://openrouter.ai/api/v1/models"]);
    let (models_exit, models_stdout, models_stderr) = command_output_text(models_cmd)?;
    let models_json: Value = serde_json::from_str(&models_stdout).unwrap_or_else(
        |_| json!({"parse_error": redact(&models_stderr), "raw_len": models_stdout.len()}),
    );
    let model_count = models_json
        .get("data")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let has_openai = models_json
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().any(|m| {
                m.get("id")
                    .and_then(Value::as_str)
                    .map(|id| id.starts_with("openai/"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    let has_anthropic = models_json
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().any(|m| {
                m.get("id")
                    .and_then(Value::as_str)
                    .map(|id| id.starts_with("anthropic/"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    let key_probe = if has_key {
        let mut curl = Command::new("curl");
        curl.args([
            "-sS",
            "-o",
            "-",
            "-w",
            "\nHTTP_CODE:%{http_code}\n",
            "--config",
            "-",
            "https://openrouter.ai/api/v1/key",
        ]);
        remove_secret_env(&mut curl);
        let mut child = curl
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("curl stdin"))?;
            let key = env::var("OPENROUTER_API_KEY").unwrap_or_default();
            writeln!(stdin, "header = \"Authorization: Bearer {key}\"")?;
        }
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let http_code = stdout
            .lines()
            .find_map(|line| line.strip_prefix("HTTP_CODE:"))
            .unwrap_or("unknown")
            .to_string();
        let body = stdout
            .lines()
            .filter(|line| !line.starts_with("HTTP_CODE:"))
            .collect::<Vec<_>>()
            .join("\n");
        let parsed: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
        let data = parsed.get("data").unwrap_or(&Value::Null);
        json!({
            "attempted": true,
            "http_code": http_code,
            "exit_code": output.status.code().unwrap_or(1),
            "valid": http_code == "200",
            "safe_account_fields": {
                "limit": data.get("limit").cloned().unwrap_or(Value::Null),
                "limit_reset": data.get("limit_reset").cloned().unwrap_or(Value::Null),
                "limit_remaining": data.get("limit_remaining").cloned().unwrap_or(Value::Null),
                "usage": data.get("usage").cloned().unwrap_or(Value::Null),
                "usage_monthly": data.get("usage_monthly").cloned().unwrap_or(Value::Null),
                "is_free_tier": data.get("is_free_tier").cloned().unwrap_or(Value::Null),
                "is_management_key": data.get("is_management_key").cloned().unwrap_or(Value::Null),
                "is_provisioning_key": data.get("is_provisioning_key").cloned().unwrap_or(Value::Null),
                "expires_at": data.get("expires_at").cloned().unwrap_or(Value::Null)
            },
            "stderr_redacted": redact(&stderr),
            "secret_printed": false
        })
    } else {
        json!({
            "attempted": false,
            "valid": false,
            "missing_env": "OPENROUTER_API_KEY",
            "secret_printed": false
        })
    };

    let responses_probe = if has_key {
        let body = json!({"model": model, "input": prompt, "max_output_tokens": 64}).to_string();
        let mut curl = Command::new("curl");
        curl.args([
            "-sS",
            "-o",
            "-",
            "-w",
            "\nHTTP_CODE:%{http_code}\n",
            "--config",
            "-",
            "-H",
            "content-type: application/json",
            "-d",
            &body,
            "https://openrouter.ai/api/v1/responses",
        ]);
        remove_secret_env(&mut curl);
        let mut child = curl
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("curl stdin"))?;
            let key = env::var("OPENROUTER_API_KEY").unwrap_or_default();
            writeln!(stdin, "header = \"Authorization: Bearer {key}\"")?;
        }
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let http_code = stdout
            .lines()
            .find_map(|line| line.strip_prefix("HTTP_CODE:"))
            .unwrap_or("unknown")
            .to_string();
        json!({
            "attempted": true,
            "authenticated": true,
            "http_code": http_code,
            "exit_code": output.status.code().unwrap_or(1),
            "response_redacted_preview": redact(&stdout.lines().take(5).collect::<Vec<_>>().join("\\n")),
            "stderr_redacted": redact(&stderr),
        })
    } else {
        let mut unauth = Command::new("curl");
        unauth.args([
            "-sS",
            "-o",
            "-",
            "-w",
            "\nHTTP_CODE:%{http_code}\n",
            "-H",
            "content-type: application/json",
            "-d",
            &json!({"model": model, "input": "ping"}).to_string(),
            "https://openrouter.ai/api/v1/responses",
        ]);
        let (exit_code, stdout, stderr) = command_output_text(unauth)?;
        let http_code = stdout
            .lines()
            .find_map(|line| line.strip_prefix("HTTP_CODE:"))
            .unwrap_or("unknown")
            .to_string();
        json!({
            "attempted": true,
            "authenticated": false,
            "missing_env": "OPENROUTER_API_KEY",
            "http_code": http_code,
            "exit_code": exit_code,
            "response_redacted_preview": redact(&stdout.lines().take(5).collect::<Vec<_>>().join("\\n")),
            "stderr_redacted": redact(&stderr),
        })
    };

    let response_http = responses_probe
        .get("http_code")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let response_preview = responses_probe
        .get("response_redacted_preview")
        .and_then(Value::as_str)
        .unwrap_or("");
    let key_valid = key_probe
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let generation_ok = response_http.starts_with('2');
    let account_policy_blocked = response_http == "404"
        && (response_preview.contains("guardrail restrictions")
            || response_preview.contains("settings/privacy"));
    let ok = models_exit == 0 && model_count > 0 && has_openai && has_anthropic;
    let out = json!({
        "ok": ok,
        "enabled": full_access_granted(),
        "decision_id": if full_access_granted() {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
        "provider": "openrouter",
        "base_url": "https://openrouter.ai/api/v1",
        "responses_url": "https://openrouter.ai/api/v1/responses",
        "models_exit_code": models_exit,
        "model_count": model_count,
        "has_openai_models": has_openai,
        "has_anthropic_models": has_anthropic,
        "target_model": model,
        "key_probe": key_probe,
        "responses_probe": responses_probe,
        "authenticated_generation_ok": generation_ok,
        "authenticated_key_valid": key_valid,
        "account_policy_blocked": account_policy_blocked,
        "secret_printed": false
    });
    fs::create_dir_all(state_dir())?;
    fs::write(
        state_dir().join("openrouter-proof.json"),
        serde_json::to_vec_pretty(&out)?,
    )?;
    append_ledger(
        "network.jsonl",
        json!({"event":"openrouter_probe","decision": if ok {"allow"} else {"deny"},"result":out}),
    )?;
    Ok(out)
}

pub fn claude_bridge_value(prompt: Option<&str>, execute: bool) -> Result<Value> {
    claude_bridge_value_with_auth(prompt, execute, false)
}

pub fn claude_bridge_value_with_auth(
    prompt: Option<&str>,
    execute: bool,
    allow_default_auth: bool,
) -> Result<Value> {
    let claude = which("claude");
    let version = claude.as_ref().and_then(|_| {
        let mut cmd = Command::new("claude");
        cmd.arg("--version");
        command_output_text(cmd)
            .ok()
            .map(|(_, stdout, stderr)| redact(&(stdout + &stderr)).trim().to_string())
    });
    let mut result = json!({
        "ok": claude.is_some(),
        "enabled": full_access_granted(),
        "decision_id": if full_access_granted() {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
        "claude_path": claude.map(|p| p.display().to_string()),
        "version": version,
        "mode": "supervised-external-cli-bridge",
        "auth_mode": if allow_default_auth {"claude-default-auth-no-env-secrets"} else {"bare-env-only"},
        "secret_printed": false,
        "executed": false
    });
    if execute {
        let prompt = prompt.unwrap_or("Return compact JSON: {\"bridge\":\"ok\"}.");
        let prompt_hash = sha256_bytes(prompt.as_bytes());
        let mut cmd = Command::new("claude");
        if !allow_default_auth {
            cmd.arg("--bare");
        }
        cmd.args([
            "--print",
            "--output-format",
            "json",
            "--permission-mode",
            "plan",
            "--no-session-persistence",
            prompt,
        ]);
        let (code, stdout, stderr) = command_output_text(cmd)?;
        result["executed"] = json!(true);
        result["exit_code"] = json!(code);
        result["prompt_hash"] = json!(prompt_hash);
        result["stdout_redacted_preview"] = json!(redact(
            &stdout.lines().take(8).collect::<Vec<_>>().join("\\n")
        ));
        result["stderr_redacted_preview"] = json!(redact(
            &stderr.lines().take(8).collect::<Vec<_>>().join("\\n")
        ));
        result["ok"] = json!(code == 0);
    }
    append_ledger(
        "network.jsonl",
        json!({"event":"claude_bridge","decision": if result.get("ok").and_then(Value::as_bool).unwrap_or(false) {"allow"} else {"deny"},"result":result}),
    )?;
    Ok(result)
}

pub fn browser_computer_value() -> Result<Value> {
    let mut cmd = Command::new("codex");
    cmd.args(["features", "list"]);
    let (features_exit_code, text, features_stderr) =
        command_output_text(cmd).unwrap_or_else(|err| (1, String::new(), err.to_string()));
    let has_browser = text
        .lines()
        .any(|line| line.starts_with("browser_use") && line.contains("true"));
    let has_browser_external = text
        .lines()
        .any(|line| line.starts_with("browser_use_external") && line.contains("true"));
    let has_computer = text
        .lines()
        .any(|line| line.starts_with("computer_use") && line.contains("true"));
    let os = env::consts::OS;
    let linux_computer_caveat = os == "linux";
    let out = json!({
        "ok": has_browser && has_computer,
        "enabled": full_access_granted(),
        "decision_id": if full_access_granted() {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
        "codex_features": {
            "browser_use": has_browser,
            "browser_use_external": has_browser_external,
            "computer_use": has_computer
        },
        "features_exit_code": features_exit_code,
        "features_stderr_redacted": redact(&features_stderr),
        "platform": os,
        "linux_computer_use_doc_caveat": linux_computer_caveat,
        "policy": {
            "auth_flows": "allowed only by explicit task scope",
            "secret_entry": "forbidden",
            "screenshots": "redact/review before ledger",
            "max_computer_agents": 1
        }
    });
    append_ledger(
        "browser-computer.jsonl",
        json!({"event":"browser_computer_enablement","decision": if out.get("ok").and_then(Value::as_bool).unwrap_or(false) {"allow"} else {"deny"},"result":out}),
    )?;
    Ok(out)
}

fn memory_root() -> PathBuf {
    env::var_os("CODEX_HARNESS_MEMORY_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/flexnetos/.codex/memories"))
}

fn collect_memory_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let lower = path.to_string_lossy().to_ascii_lowercase();
        if secret_path_violation(&lower) {
            continue;
        }
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            collect_memory_files(&path, out)?;
        } else if meta.is_file()
            && matches!(
                path.extension().and_then(OsStr::to_str),
                Some("md" | "jsonl" | "txt")
            )
        {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

pub fn memory_audit_value() -> Result<Value> {
    let root = memory_root();
    let mut files = Vec::new();
    collect_memory_files(&root, &mut files)?;
    let mut redacted_lines = 0usize;
    let mut total_lines = 0usize;
    let mut total_bytes = 0u64;
    for file in &files {
        let meta = fs::metadata(file)?;
        total_bytes += meta.len();
        let fh = fs::File::open(file)?;
        for line in io::BufReader::new(fh).lines() {
            let line = line?;
            total_lines += 1;
            if redact(&line) != line {
                redacted_lines += 1;
            }
        }
    }
    let out = json!({
        "ok": true,
        "memory_root": root.display().to_string(),
        "files": files.len(),
        "total_lines": total_lines,
        "total_bytes": total_bytes,
        "redaction_hits": redacted_lines,
        "raw_secret_paths_read": false,
    });
    append_ledger(
        "memory.jsonl",
        json!({"event":"memory_audit","decision":"allow","result":out}),
    )?;
    Ok(out)
}

pub fn memory_export_redacted(limit: Option<usize>) -> Result<Value> {
    let root = memory_root();
    let mut files = Vec::new();
    collect_memory_files(&root, &mut files)?;
    let mut exported = 0usize;
    let max = limit.unwrap_or(usize::MAX);
    for file in &files {
        if exported >= max {
            break;
        }
        let fh = fs::File::open(file)?;
        for (idx, line) in io::BufReader::new(fh).lines().enumerate() {
            if exported >= max {
                break;
            }
            let line = line?;
            println!(
                "{}",
                canonical_json(&json!({
                    "path": file.display().to_string(),
                    "line": idx + 1,
                    "text": redact(&line),
                }))
            );
            exported += 1;
        }
    }
    let out = json!({"ok": true, "exported_lines": exported, "redacted": true});
    append_ledger(
        "memory.jsonl",
        json!({"event":"memory_export_redacted","decision":"allow","result":out}),
    )?;
    Ok(out)
}

pub fn memory_disable_plan_value() -> Result<Value> {
    let out = json!({
        "ok": true,
        "mutation": false,
        "plan": [
            "keep JSONL ledgers canonical",
            "do not delete memory files automatically",
            "if disabling is approved later, archive memory config first",
            "write an explicit disable decision id before changing runtime config",
            "verify codex-harness-status counters and codex features after any approved change"
        ],
    });
    append_ledger(
        "memory.jsonl",
        json!({"event":"memory_disable_plan","decision":"plan_only","result":out}),
    )?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn policy_denies_rm() {
        let d = policy_decision(&["rm".into(), "-rf".into(), "/tmp/x".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
    }

    #[test]
    fn policy_denies_ledger_write() {
        let d = policy_decision(&[
            "tee".into(),
            "/home/flexnetos/lifeos/src/envctl/home/agent-env/codex-harness/ledger/harness.jsonl"
                .into(),
        ]);
        assert_eq!(d.decision, DecisionKind::Deny);
    }

    #[test]
    fn policy_denies_nested_codex() {
        let d = policy_decision(&["codex".into(), "exec".into(), "hi".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
    }

    #[test]
    fn policy_allows_pwd() {
        let d = policy_decision(&["pwd".into()]);
        assert_eq!(d.decision, DecisionKind::Allow);
    }

    #[test]
    fn hooks_pretool_denies_codex() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        fs::create_dir_all(dir.path().join("codex-harness/ledger")).unwrap();
        let input =
            json!({"hook_event_name":"PreToolUse","tool_input":{"argv":["codex","exec","x"]}});
        let resp = hook_response(&input).unwrap();
        assert_eq!(
            resp.pointer("/hookSpecificOutput/permissionDecision")
                .and_then(Value::as_str),
            Some("deny")
        );
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn hooks_deny_depth_two_subagent() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        fs::create_dir_all(dir.path().join("codex-harness/ledger")).unwrap();
        let input = json!({"hook_event_name":"SubagentStart","depth":2});
        let resp = hook_response(&input).unwrap();
        assert_eq!(
            resp.pointer("/hookSpecificOutput/permissionDecision")
                .and_then(Value::as_str),
            Some("deny")
        );
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn ledger_hash_chain_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        fs::create_dir_all(dir.path().join("codex-harness/ledger")).unwrap();
        append_ledger("harness.jsonl", json!({"event":"test","decision":"allow"})).unwrap();
        append_ledger("harness.jsonl", json!({"event":"test2","decision":"allow"})).unwrap();
        assert_eq!(
            verify_ledger(&dir.path().join("codex-harness/ledger/harness.jsonl")).unwrap(),
            2
        );
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn redaction_masks_secrets() {
        assert_eq!(
            redact("cat ~/.codex/auth.json"),
            "[REDACTED:SENSITIVE-COMMAND]"
        );
        assert_eq!(redact("OPENAI_API_KEY=abc"), "[REDACTED:SENSITIVE-COMMAND]");
    }

    #[test]
    fn cross_platform_path_cases_archive_encoding() {
        assert_eq!(archive_encode_path(Path::new("/a/b")), "%2Fa%2Fb");
        assert!(archive_encode_path(Path::new("C:\\Users\\x")).contains("%5C"));
    }

    #[test]
    fn policy_denies_unknown_harness_prefix() {
        let d = policy_decision(&["codex-harness-evil".into(), "--".into(), "rm".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
        assert_eq!(d.violation.as_deref(), Some("unknown_harness_command"));
    }

    #[test]
    fn process_supervisor_spawn_limit_policy() {
        let d = policy_decision(&["nohup".into(), "sleep".into(), "99".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
    }

    #[test]
    fn policy_denies_danger_without_decision_id() {
        let d = policy_decision(&["codex".into(), "--danger-full-access".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
        assert_eq!(d.violation.as_deref(), Some("danger_without_decision_id"));
    }

    #[test]
    fn policy_prompts_danger_with_decision_id() {
        let d = policy_decision(&[
            "tool".into(),
            "--danger-full-access".into(),
            "--decision-id".into(),
            "DEC-1".into(),
        ]);
        assert_eq!(d.decision, DecisionKind::Prompt);
    }

    #[test]
    fn policy_denies_github_mutation_without_guard() {
        let d = policy_decision(&["gh".into(), "pr".into(), "merge".into(), "1".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
        assert_eq!(
            d.violation.as_deref(),
            Some("github_mutation_without_guard")
        );
    }

    #[test]
    fn model_router_marker_unlocks_subagent_depth_one() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        fs::create_dir_all(dir.path().join("codex-harness/ledger")).unwrap();
        route_model_tasks(&["containment policy tests".to_string()]).unwrap();
        let input = json!({"hook_event_name":"SubagentStart","depth":1});
        let resp = hook_response(&input).unwrap();
        assert!(resp.as_object().map(|o| o.is_empty()).unwrap_or(false));
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn redb_index_integrity_uses_jsonl_source() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        fs::create_dir_all(dir.path().join("codex-harness/ledger")).unwrap();
        append_ledger(
            "harness.jsonl",
            json!({"event":"index-test","decision":"allow"}),
        )
        .unwrap();
        let out = index_integrity_check().unwrap();
        assert_eq!(out.get("backend").and_then(Value::as_str), Some("redb"));
        assert_eq!(
            out.get("sqlite_default").and_then(Value::as_bool),
            Some(false)
        );
        env::remove_var("CODEX_HARNESS_ROOT");
    }
}
