#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
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
        "bearer ",
        "authorization:",
        "private key",
        "id_rsa",
        "id_ed25519",
        "token",
        "secret",
        "password",
    ];
    if sensitive_markers.iter().any(|m| lower.contains(m)) {
        out = "[REDACTED:SENSITIVE-COMMAND]".to_string();
    }
    for key in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "GITHUB_TOKEN",
        "GH_TOKEN",
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
        "ANTHROPIC_API_KEY",
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

    let deny = |reason: &str| PolicyDecision {
        decision: DecisionKind::Deny,
        reason: reason.to_string(),
        redacted_preview: preview.clone(),
    };
    let allow = |reason: &str| PolicyDecision {
        decision: DecisionKind::Allow,
        reason: reason.to_string(),
        redacted_preview: preview.clone(),
    };
    let prompt = |reason: &str| PolicyDecision {
        decision: DecisionKind::Prompt,
        reason: reason.to_string(),
        redacted_preview: preview.clone(),
    };

    if argv.is_empty() {
        return deny("empty command");
    }
    if preview.contains("[REDACTED:SENSITIVE-COMMAND]") {
        return deny("secret-bearing command or path is forbidden");
    }
    if lower.contains("codex-harness/ledger")
        || lower.contains("/ledger/")
        || lower.contains("agent-env/archive")
        || lower.contains("/archive/")
    {
        return deny("direct ledger/archive mutation is forbidden; use sanctioned harness flow");
    }
    if ["rm", "unlink", "rmdir"].contains(&first.as_str()) {
        return deny("delete command forbidden; use archive flow");
    }
    if first == "git" {
        if argv.iter().any(|a| a == "clean")
            || (argv.iter().any(|a| a == "reset") && argv.iter().any(|a| a == "--hard"))
            || (argv.iter().any(|a| a == "branch") && argv.iter().any(|a| a == "-D"))
            || (argv.iter().any(|a| a == "push")
                && argv
                    .iter()
                    .any(|a| a == "--force" || a == "--force-with-lease"))
        {
            return deny("destructive git command forbidden");
        }
        if argv.iter().any(|a| a == "fetch") {
            return prompt("network git fetch requires approval");
        }
        return allow("read-only git command allowed");
    }
    if ["codex", "claude", "ollama"].contains(&first.as_str()) {
        if argv.iter().any(|a| a.contains("codex-harness-runner")) {
            return allow("supervised agent/model command");
        }
        return deny("nested codex/claude/ollama must go through codex-harness-runner");
    }
    if ["tmux", "screen", "nohup", "disown", "setsid"].contains(&first.as_str())
        || lower.contains(" &")
        || lower.contains("start-job")
        || lower.contains("start-process")
    {
        return deny("background escape must go through codex-harness-runner");
    }
    if (lower.contains("curl") || lower.contains("wget"))
        && (lower.contains("| sh") || lower.contains("iex"))
    {
        return deny("curl-pipe-shell installer forbidden");
    }
    if lower.contains("npm install -g") && lower.contains("codex") {
        return deny("non-Nix Codex install forbidden");
    }
    if lower.contains("pip install") && lower.contains("openai-codex") {
        return deny("Python Codex SDK cannot be production harness runtime");
    }
    if lower.contains("danger-full-access")
        || lower.contains("bypass")
        || lower.contains("yolo")
        || lower.contains("ignore-rules")
    {
        return deny("bypass, danger, yolo, or ignore-rules invocation forbidden");
    }
    let read_only = [
        "pwd",
        "ls",
        "cat",
        "sed",
        "rg",
        "find",
        "fd",
        "nix",
        "codex-harness-status",
        "codex-harness-nix-verify",
        "codex-harness-audit",
        "jq",
        "sleep",
    ];
    if read_only.contains(&first.as_str()) {
        return allow("read-only command allowlisted");
    }
    if first == "cargo" {
        if argv.iter().any(|a| a == "build") {
            return prompt("cargo build requires explicit approved workspace");
        }
        return allow("cargo check/test/fmt/clippy allowed in approved harness workspace");
    }
    if ["gh", "nix-build", "nix"].contains(&first.as_str()) {
        return prompt("network or build command requires approval");
    }
    prompt("command not explicitly allowlisted")
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
            append_ledger(
                "decisions.jsonl",
                json!({"event":"subagent_depth_deny","decision":"deny","reason":"max_depth=1","depth":depth}),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness max_depth=1 denies depth > 1"},"systemMessage":"codex-harness denied subagent depth > 1"}),
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
        if lower.contains("disable hooks")
            || lower.contains("danger-full-access")
            || lower.contains("bypass")
            || lower.contains("leak secrets")
            || lower.contains("without archive")
        {
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
        if lower.contains("danger") || lower.contains("bypass") || lower.contains("auth.json") {
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
    let file = fs::File::open(path)?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(&line)?;
        if v.get("source").and_then(Value::as_str) == Some(source) {
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
        .map(|p| p == "/home/flexnetos/.nix-profile/bin/codex")
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
        .env_remove("ANTHROPIC_API_KEY")
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
    for name in [
        "harness.jsonl",
        "processes.jsonl",
        "archive.jsonl",
        "budget.jsonl",
        "decisions.jsonl",
        "research.jsonl",
    ] {
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
    fn process_supervisor_spawn_limit_policy() {
        let d = policy_decision(&["nohup".into(), "sleep".into(), "99".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
    }
}
