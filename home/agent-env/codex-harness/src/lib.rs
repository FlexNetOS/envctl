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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_PROJECT_ROOT: &str = "/home/flexnetos/meta/src/envctl/home";
pub const DEFAULT_HARNESS_ROOT: &str = "/home/flexnetos/meta/src/envctl/home/agent-env";
pub const PROVIDER_RECEIPT_MAX_AGE: Duration = Duration::from_secs(15 * 60);

const COMPILED_PROVIDER_SOURCE: &[u8] = include_bytes!("lib.rs");
const COMPILED_PROVIDER_CONFIG: &[u8] = include_bytes!("../config/policy/providers.toml");

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
pub const SESSION_CAPABILITIES: &[&str] = &[
    "external_providers",
    "local_models",
    "network",
    "github_mutation",
    "browser_computer",
    "subagents",
    "background_jobs",
];

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

fn tracked_full_access_policy_granted_at(path: &Path) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(policy) = toml::from_str::<toml::Value>(&text) else {
        return false;
    };
    let grants = policy.get("permission_grants");
    policy
        .get("full_access_decision_id")
        .and_then(toml::Value::as_str)
        == Some(USER_FULL_ACCESS_DECISION_ID)
        && grants
            .and_then(|value| value.get("decision_id"))
            .and_then(toml::Value::as_str)
            == Some(USER_FULL_ACCESS_DECISION_ID)
        && grants
            .and_then(|value| value.get("operator_grants_are_execution_context"))
            .and_then(toml::Value::as_bool)
            == Some(true)
        && grants
            .and_then(|value| value.get("expanded_access_is_not_a_blocker"))
            .and_then(toml::Value::as_bool)
            == Some(true)
        && grants
            .and_then(|value| value.get("danger_full_access"))
            .and_then(toml::Value::as_str)
            == Some("keep")
}

pub fn tracked_full_access_policy_granted() -> bool {
    tracked_full_access_policy_granted_at(&codex_harness_dir().join("policy/policy.toml"))
}

pub fn live_permission_profile() -> Option<String> {
    env::var("CODEX_PERMISSION_PROFILE")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn session_key() -> String {
    env::var("CODEX_THREAD_ID")
        .unwrap_or_else(|_| "standalone".to_string())
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn session_capability_path() -> PathBuf {
    state_dir()
        .join("sessions")
        .join(session_key())
        .join("capabilities.json")
}

fn read_session_capability_overrides() -> Result<BTreeMap<String, bool>> {
    let path = session_capability_path();
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read session capability state {}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("parse session capability state {}", path.display()))?;
    let entries = value
        .get("overrides")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("session capability state has no overrides object"))?;
    entries
        .iter()
        .map(|(name, value)| {
            value
                .as_bool()
                .map(|enabled| (name.clone(), enabled))
                .ok_or_else(|| anyhow!("session capability override {name} is not boolean"))
        })
        .collect()
}

pub fn session_capability_enabled(capability: &str) -> bool {
    SESSION_CAPABILITIES.contains(&capability)
        && read_session_capability_overrides()
            .map(|overrides| overrides.get(capability).copied().unwrap_or(true))
            .unwrap_or(false)
}

pub fn session_capability_status() -> Value {
    let overrides_result = read_session_capability_overrides();
    let state_error = overrides_result
        .as_ref()
        .err()
        .map(|error| redact(&error.to_string()));
    let overrides = overrides_result.unwrap_or_default();
    let capabilities = SESSION_CAPABILITIES
        .iter()
        .map(|capability| {
            (
                (*capability).to_string(),
                json!({
                    "enabled": session_capability_enabled(capability),
                    "session_override": overrides.get(*capability),
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();
    json!({
        "ok": true,
        "schema": "codex-harness.session-capabilities.v1",
        "thread_id": session_key(),
        "permission_profile_signal": live_permission_profile(),
        "codex_os_permissions_are_authoritative": true,
        "capabilities_are_optional_routing_switches": true,
        "state_valid": state_error.is_none(),
        "state_error": state_error,
        "tracked_operator_policy": tracked_full_access_policy_granted(),
        "operator_full_access_intent_recorded": tracked_full_access_policy_granted(),
        "live_permission_effective": "unknown-to-child-process; use /permissions",
        "capabilities": capabilities,
        "hard_safety_not_toggleable": [
            "secret reads or secret output",
            "destructive user-data deletion",
            "force-push",
            "direct ledger or archive mutation"
        ]
    })
}

pub fn set_session_capability(capability: &str, enabled: bool) -> Result<Value> {
    if !SESSION_CAPABILITIES.contains(&capability) {
        return Err(anyhow!(
            "unknown capability {capability}; expected one of {}",
            SESSION_CAPABILITIES.join(", ")
        ));
    }
    let mut overrides = read_session_capability_overrides()?;
    overrides.insert(capability.to_string(), enabled);
    let path = session_capability_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "schema": "codex-harness.session-capabilities.v1",
            "thread_id": session_key(),
            "updated_utc": utc_now(),
            "overrides": overrides,
        }))?,
    )?;
    append_ledger(
        "decisions.jsonl",
        json!({
            "event": "session_capability_toggle",
            "decision": if enabled {"enable"} else {"disable"},
            "capability": capability,
            "thread_id": session_key(),
            "effective": session_capability_enabled(capability),
        }),
    )?;
    Ok(session_capability_status())
}

pub fn set_session_capability_preset(preset: &str) -> Result<Value> {
    let enabled = match preset {
        "full" => true,
        "restricted" => false,
        _ => {
            return Err(anyhow!(
                "unknown preset {preset}; expected full or restricted"
            ))
        }
    };
    let mut overrides = BTreeMap::new();
    for capability in SESSION_CAPABILITIES {
        overrides.insert((*capability).to_string(), enabled);
    }
    let path = session_capability_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "schema": "codex-harness.session-capabilities.v1",
            "thread_id": session_key(),
            "updated_utc": utc_now(),
            "preset": preset,
            "overrides": overrides,
        }))?,
    )?;
    append_ledger(
        "decisions.jsonl",
        json!({
            "event": "session_capability_preset",
            "decision": preset,
            "thread_id": session_key(),
        }),
    )?;
    Ok(session_capability_status())
}

pub fn record_full_access_receipt(reason: &str) -> Result<Value> {
    fs::create_dir_all(state_dir())?;
    let value = json!({
        "ok": true,
        "authority": "receipt-only; live /permissions controls access",
        "decision_id": USER_FULL_ACCESS_DECISION_ID,
        "ts_utc": utc_now(),
        "reason": redact(reason),
        "permission_profile": live_permission_profile(),
        "operator_full_access_intent_recorded": tracked_full_access_policy_granted(),
        "live_permission_effective": "unknown-to-child-process",
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
        json!({"event":"full_access_receipt","decision":"record","decision_id":USER_FULL_ACCESS_DECISION_ID,"reason":redact(reason),"operator_intent_recorded":tracked_full_access_policy_granted()}),
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

pub fn provider_contract_fingerprint(provider: &str) -> Result<String> {
    let source_path = codex_harness_dir().join("src/lib.rs");
    let config_path = codex_harness_dir().join("config/policy/providers.toml");
    let source = fs::read(&source_path)
        .with_context(|| format!("read provider source {}", source_path.display()))?;
    let config = fs::read(&config_path)
        .with_context(|| format!("read provider config {}", config_path.display()))?;
    if source != COMPILED_PROVIDER_SOURCE {
        return Err(anyhow!(
            "provider source differs from the source embedded in this binary"
        ));
    }
    if config != COMPILED_PROVIDER_CONFIG {
        return Err(anyhow!(
            "provider config differs from the config embedded in this binary"
        ));
    }
    let mut hasher = Sha256::new();
    hasher.update(b"codex-harness.provider-contract.v1\0");
    hasher.update(provider.as_bytes());
    hasher.update(b"\0");
    hasher.update(&source);
    hasher.update(b"\0");
    hasher.update(&config);
    Ok(hex::encode(hasher.finalize()))
}

fn parse_utc_timestamp(timestamp: &str) -> Option<i64> {
    if timestamp.len() != 20
        || timestamp.as_bytes().get(4) != Some(&b'-')
        || timestamp.as_bytes().get(7) != Some(&b'-')
        || timestamp.as_bytes().get(10) != Some(&b'T')
        || timestamp.as_bytes().get(13) != Some(&b':')
        || timestamp.as_bytes().get(16) != Some(&b':')
        || timestamp.as_bytes().get(19) != Some(&b'Z')
    {
        return None;
    }
    let parse = |start: usize, end: usize| timestamp.get(start..end)?.parse::<i64>().ok();
    let year = parse(0, 4)?;
    let month = parse(5, 7)?;
    let day = parse(8, 10)?;
    let hour = parse(11, 13)?;
    let minute = parse(14, 16)?;
    let second = parse(17, 19)?;
    if !(1..=12).contains(&month)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days_in_month).contains(&day) {
        return None;
    }

    let adjusted_year = year - i64::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;
    Some(days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn provider_receipt_valid_at(
    entries: &[Value],
    event: &str,
    provider: &str,
    require_executed: bool,
    expected_fingerprint: &str,
    now_unix: i64,
    max_age: Duration,
) -> bool {
    let latest = entries.iter().rev().find(|entry| {
        entry.get("event").and_then(Value::as_str) == Some(event)
            && (!require_executed
                || entry.pointer("/result/executed").and_then(Value::as_bool) == Some(true))
    });
    let Some(entry) = latest else {
        return false;
    };
    let succeeded = entry.get("decision").and_then(Value::as_str) == Some("allow")
        && (entry.get("exit_code").and_then(Value::as_i64) == Some(0)
            || entry.pointer("/result/exit_code").and_then(Value::as_i64) == Some(0))
        && entry
            .pointer("/result/ok")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        && (!require_executed
            || entry.pointer("/result/executed").and_then(Value::as_bool) == Some(true));
    let identity_matches = entry.get("provider").and_then(Value::as_str) == Some(provider)
        && entry
            .get("provider_contract_fingerprint")
            .and_then(Value::as_str)
            == Some(expected_fingerprint);
    let timestamp = entry
        .get("provider_receipt_ts_utc")
        .and_then(Value::as_str)
        .and_then(parse_utc_timestamp);
    let fresh = timestamp
        .and_then(|timestamp| now_unix.checked_sub(timestamp))
        .map(|age| age >= 0 && age <= max_age.as_secs() as i64)
        .unwrap_or(false);
    succeeded && identity_matches && fresh
}

pub fn provider_receipt_current(entry: &Value, provider: &str) -> bool {
    let Ok(expected_fingerprint) = provider_contract_fingerprint(provider) else {
        return false;
    };
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok());
    let Some(now_unix) = now_unix else {
        return false;
    };
    let identity_matches = entry.get("provider").and_then(Value::as_str) == Some(provider)
        && entry
            .get("provider_contract_fingerprint")
            .and_then(Value::as_str)
            == Some(expected_fingerprint.as_str());
    let fresh = entry
        .get("provider_receipt_ts_utc")
        .and_then(Value::as_str)
        .and_then(parse_utc_timestamp)
        .and_then(|timestamp| now_unix.checked_sub(timestamp))
        .map(|age| age >= 0 && age <= PROVIDER_RECEIPT_MAX_AGE.as_secs() as i64)
        .unwrap_or(false);
    identity_matches && fresh
}

pub fn latest_provider_receipt_valid(
    path: &Path,
    event: &str,
    provider: &str,
    require_executed: bool,
) -> bool {
    let Ok(expected_fingerprint) = provider_contract_fingerprint(provider) else {
        return false;
    };
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let entries = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect::<Vec<_>>();
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    provider_receipt_valid_at(
        &entries,
        event,
        provider,
        require_executed,
        &expected_fingerprint,
        now.as_secs() as i64,
        PROVIDER_RECEIPT_MAX_AGE,
    )
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

    if first == "codex"
        && argv
            .iter()
            .any(|argument| argument == "--dangerously-bypass-approvals-and-sandbox")
    {
        return allow("the explicit Codex full-access frontdoor is delegated to live /permissions");
    }

    if nested_model_provider(&first, &lower) {
        return deny(
            "nested Codex/Claude/model provider launch must go through codex-harness-runner and model-router",
            "nested_model_provider_without_runner",
        );
    }

    if browser_or_computer_use(&first, &lower) {
        if session_capability_enabled("browser_computer") {
            return allow("browser/computer-use allowed by the optional session routing switch");
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
            if session_capability_enabled("network") {
                return allow("git network command allowed by live session capability");
            }
            return deny(
                "unmanaged git network command is forbidden by containment",
                "unmanaged_network",
            );
        }
        return allow("read-only git command allowed");
    }

    if first == "gh" {
        if gh_mutation(argv) {
            return deny(
                "GitHub mutation requires codex-harness-github-guard",
                "github_mutation_without_guard",
            );
        }
        if session_capability_enabled("network") {
            return allow("GitHub read allowed by live network capability");
        }
        return prompt(
            "GitHub read command uses network; run only as explicit foreground proof",
            Some("network_read"),
        );
    }

    if unmanaged_network_requested(&first, &lower) {
        if session_capability_enabled("network") {
            return allow("network command allowed by live session capability");
        }
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

fn force_push_requested(argv: &[String]) -> bool {
    basename(argv.first().map(String::as_str).unwrap_or_default()) == "git"
        && has_arg(argv, &["push"])
        && argv.iter().any(|arg| {
            matches!(
                arg.as_str(),
                "--force" | "-f" | "--force-with-lease" | "--force-if-includes" | "--mirror"
            ) || arg.starts_with("--force=")
                || arg.starts_with("--force-with-lease=")
                || arg.starts_with('+')
        })
}

fn git_network(argv: &[String]) -> bool {
    has_arg(argv, &["push", "fetch", "pull", "clone", "ls-remote"])
        || (has_arg(argv, &["submodule"]) && has_arg(argv, &["update", "sync"]))
        || (has_arg(argv, &["remote"]) && has_arg(argv, &["update", "prune"]))
        || argv
            .iter()
            .any(|arg| arg == "--upload-pack" || arg.starts_with("--upload-pack="))
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
        if !session_capability_enabled("subagents") {
            record_bad_behavior(
                "subagent_session_disabled",
                "subagents disabled for this chat session",
                &redact(&input.to_string()),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness session capability disables subagents"},"systemMessage":"codex-harness denied subagent spawn because the session toggle is off"}),
            );
        }
        if !session_capability_enabled("network") {
            record_bad_behavior(
                "subagent_network_disabled",
                "network disabled for this chat session",
                &redact(&input.to_string()),
            )?;
            return Ok(
                json!({"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"codex-harness session capability disables network for subagents"},"systemMessage":"codex-harness denied subagent spawn because the current network toggle is off"}),
            );
        }
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
        if lower.contains("leak secrets") || lower.contains("without archive") {
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
        if lower.contains("auth.json") {
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
    let profile_frontdoor = Path::new("/home/flexnetos/.nix-profile/bin/codex");
    let profile_realpath = fs::canonicalize(profile_frontdoor).ok();
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
    let approved_frontdoor = codex_path
        .as_ref()
        .map(|path| approved_codex_frontdoor_path(path))
        .unwrap_or(false);
    let profile_owned = approved_frontdoor && realpath.is_some() && realpath == profile_realpath;
    let store_owned = realpath
        .as_ref()
        .map(|p| p.starts_with("/nix/store"))
        .unwrap_or(false);
    let first_shadow_ok = shadows
        .first()
        .map(PathBuf::from)
        .filter(|path| approved_codex_frontdoor_path(path))
        .and_then(|path| fs::canonicalize(path).ok())
        .zip(profile_realpath.as_ref())
        .map(|(candidate, profile)| &candidate == profile)
        .unwrap_or(false);
    json!({
        "codex_path": codex_path.map(|p| p.display().to_string()),
        "realpath": realpath.map(|p| p.display().to_string()),
        "profile_frontdoor": profile_frontdoor.display().to_string(),
        "profile_realpath": profile_realpath.map(|p| p.display().to_string()),
        "shadows": shadows,
        "profile_owned": profile_owned,
        "approved_frontdoor": approved_frontdoor,
        "store_owned": store_owned,
        "first_shadow_ok": first_shadow_ok,
        "ok": profile_owned && store_owned && first_shadow_ok
    })
}

fn approved_codex_frontdoor_path(path: &Path) -> bool {
    path.starts_with("/nix/store") || path.starts_with("/home/flexnetos/.nix-profile")
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
    if !session_capability_enabled("background_jobs") {
        return Err(anyhow!(
            "background jobs are disabled by the optional session routing switch"
        ));
    }
    let provider = match argv.first().map(|value| basename(value)).as_deref() {
        Some("claude") => Some("claude_bridge"),
        Some("ollama") => Some("ollama"),
        _ => None,
    };
    let provider_contract_fingerprint = provider.map(provider_contract_fingerprint).transpose()?;
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
        json!({"event":"spawn","decision":"allow","job_id":job.job_id,"pid":pid,"kind":job_kind(argv),"provider":provider,"provider_contract_fingerprint":provider_contract_fingerprint,"provider_receipt_ts_utc":provider.map(|_| utc_now()),"command_hash":sha256_bytes(command_preview(argv).as_bytes()),"command_preview":command_preview(argv)}),
    )?;
    Ok(job)
}

pub fn spawn_codex_exec(cwd: &Path, profile: &str, prompt: &str) -> Result<JobRecord> {
    require_model_router_ready()?;
    if !session_capability_enabled("network") || !session_capability_enabled("subagents") {
        return Err(anyhow!(
            "Codex children require the optional network and subagents routing switches"
        ));
    }
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
    require_model_router_ready()?;
    if !session_capability_enabled("local_models") {
        return Err(anyhow!(
            "local models are disabled by the optional session routing switch"
        ));
    }
    let argv = vec![
        "ollama".to_string(),
        "run".to_string(),
        model.to_string(),
        prompt.to_string(),
    ];
    spawn_supervised_unchecked(cwd, &argv)
}

fn claude_run_argv(prompt: &str, allow_default_auth: bool) -> Vec<String> {
    let mut argv = vec!["claude".to_string()];
    if !allow_default_auth {
        argv.push("--bare".to_string());
    }
    argv.extend([
        "--safe-mode".to_string(),
        "--tools".to_string(),
        String::new(),
        "--mcp-config".to_string(),
        r#"{"mcpServers":{}}"#.to_string(),
        "--strict-mcp-config".to_string(),
        "--disable-slash-commands".to_string(),
        "--no-chrome".to_string(),
        "--print".to_string(),
        "--output-format".to_string(),
        "json".to_string(),
        "--no-session-persistence".to_string(),
        prompt.to_string(),
    ]);
    argv
}

fn claude_run_command(prompt: &str, allow_default_auth: bool) -> Command {
    let argv = claude_run_argv(prompt, allow_default_auth);
    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]);
    remove_secret_env(&mut command);
    command
}

pub fn spawn_claude_run(cwd: &Path, prompt: &str, allow_default_auth: bool) -> Result<JobRecord> {
    require_model_router_ready()?;
    if !session_capability_enabled("external_providers") || !session_capability_enabled("network") {
        return Err(anyhow!(
            "Claude is disabled by optional external-provider or network routing switches"
        ));
    }
    spawn_supervised_unchecked(cwd, &claude_run_argv(prompt, allow_default_auth))
}

pub fn run_codex_exec(cwd: &Path, profile: &str, prompt: &str) -> Result<i32> {
    require_model_router_ready()?;
    if !session_capability_enabled("network") || !session_capability_enabled("subagents") {
        return Err(anyhow!(
            "Codex children require the optional network and subagents routing switches"
        ));
    }
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
    require_model_router_ready()?;
    if !session_capability_enabled("local_models") {
        return Err(anyhow!(
            "local models are disabled by the optional session routing switch"
        ));
    }
    let prompt_hash = sha256_bytes(prompt.as_bytes());
    let provider_contract_fingerprint = provider_contract_fingerprint("ollama")?;
    let mut command = Command::new("ollama");
    command.args(["run", model, prompt]).current_dir(cwd);
    remove_secret_env(&mut command);
    let output = command.output()?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", redact(&String::from_utf8_lossy(&output.stderr)));
    let code = output.status.code().unwrap_or(1);
    let provider_receipt_ts_utc = utc_now();
    append_ledger(
        "processes.jsonl",
        json!({"event":"ollama_run_complete","decision": if code == 0 {"allow"} else {"deny"},"exit_code":code,"executed":true,"provider":"ollama","provider_contract_fingerprint":provider_contract_fingerprint,"provider_receipt_ts_utc":provider_receipt_ts_utc,"model":model,"prompt_hash":prompt_hash}),
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
    let (class, profile, model, provider, reason) = if lower.contains("openrouter") {
        (
            "provider-openrouter",
            "envctl-openrouter-gpt",
            "tencent/hy3:free",
            "openrouter",
            "OpenRouter route explicitly enabled by the external-provider and network switches; authenticated proof depends on OPENROUTER_API_KEY",
        )
    } else if lower.contains("claude") {
        (
            "provider-claude-bridge",
            "envctl-claude-bridge",
            "claude-sonnet-5",
            "claude-bridge",
            "Claude direct use is routed through the supervised tool-free external CLI bridge",
        )
    } else if lower.contains("ollama")
        || lower.contains("local model")
        || lower.contains("local-model")
        || lower.contains("gemma")
        || lower.contains("qwen")
    {
        (
            "provider-local-ollama",
            "envctl-local-models",
            "gemma4:latest",
            "ollama",
            "Local model work is routed through the supervised Ollama lane",
        )
    } else if lower.contains("sol")
        || lower.contains("high-stakes")
        || lower.contains("security review")
        || lower.contains("complex coding")
    {
        (
            "openai-high-stakes",
            "envctl-gpt56-sol",
            "gpt-5.6-sol",
            "openai",
            "Sol handles high-stakes reasoning, security review, and complex coding",
        )
    } else if lower.contains("luna")
        || lower.contains("high-throughput")
        || lower.contains("mechanical")
        || lower.contains("simple")
        || lower.contains("bulk")
    {
        (
            "openai-high-throughput",
            "envctl-gpt56-luna",
            "gpt-5.6-luna",
            "openai",
            "Luna handles simple, mechanical, and high-volume tasks",
        )
    } else if lower.contains("terra")
        || lower.contains("gpt-5.6")
        || lower.contains("professional workflow")
        || lower.contains("workhorse")
    {
        (
            "openai-professional-workhorse",
            "envctl-gpt56-terra",
            "gpt-5.6-terra",
            "openai",
            "Terra is the balanced workhorse for professional workflows",
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
            "envctl-gpt56-terra",
            "gpt-5.6-terra",
            "openai",
            "model catalog/access audits use Terra plus explicit live model-access probes",
        )
    } else if lower.contains("browser") || lower.contains("computer") || lower.contains("gui") {
        (
            "browser-computer",
            "envctl-browser-computer",
            "gpt-5.6-terra",
            "openai",
            "Browser/Computer Use uses Terra and is gated through browser-computer policy",
        )
    } else if lower.contains("github") || lower.contains("gh ") {
        (
            "github-full-access",
            "envctl-github-full-access",
            "gpt-5.6-sol",
            "openai",
            "GitHub mutation uses Sol and codex-harness-github-guard with the full-access decision id",
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
    let route_enabled = match provider {
        "openrouter" | "claude-bridge" => {
            session_capability_enabled("external_providers")
                && session_capability_enabled("network")
        }
        "ollama" => session_capability_enabled("local_models"),
        _ if class == "browser-computer" => session_capability_enabled("browser_computer"),
        _ if class == "github-full-access" => {
            session_capability_enabled("github_mutation") && session_capability_enabled("network")
        }
        _ => session_capability_enabled("network") && session_capability_enabled("subagents"),
    };
    json!({
        "task": task,
        "class": class,
        "provider": provider,
        "profile": profile,
        "model": model,
        "approved_capability_expansion": route_enabled,
        "operator_intent_decision_id": if route_enabled {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
        "openrouter_enabled": session_capability_enabled("external_providers") && session_capability_enabled("network"),
        "claude_bridge_enabled": session_capability_enabled("external_providers") && session_capability_enabled("network"),
        "local_models_enabled": session_capability_enabled("local_models"),
        "browser_computer_enabled": session_capability_enabled("browser_computer"),
        "github_full_access_enabled": session_capability_enabled("github_mutation") && session_capability_enabled("network"),
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
    let all_routes_enabled = routes.iter().all(|route| {
        route
            .get("approved_capability_expansion")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });
    let value = json!({
        "ok": all_routes_enabled,
        "route_id": monotonic_id("route"),
        "ts_utc": utc_now(),
        "requires_runner": true,
        "routes": routes,
        "containment": {
            "subagent_spawn_requires_this_marker": true,
            "permission_profile": live_permission_profile(),
            "session_capabilities": session_capability_status(),
            "provider_expansion_allowed": session_capability_enabled("external_providers"),
            "operator_intent_decision_id": if tracked_full_access_policy_granted() {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
            "openrouter_shim": if session_capability_enabled("external_providers") && session_capability_enabled("network") {"enabled"} else {"disabled_by_session"},
            "claude_bridge": if session_capability_enabled("external_providers") && session_capability_enabled("network") {"enabled"} else {"disabled_by_session"},
            "local_models": if session_capability_enabled("local_models") {"enabled"} else {"disabled_by_session"},
            "browser_computer_use": if session_capability_enabled("browser_computer") {"enabled"} else {"disabled_by_session"},
            "github_full_access": if session_capability_enabled("github_mutation") && session_capability_enabled("network") {"enabled"} else {"disabled_by_session"}
        }
    });
    fs::create_dir_all(model_router_dir())?;
    fs::write(model_router_marker(), serde_json::to_vec_pretty(&value)?)?;
    append_ledger(
        "model_router.jsonl",
        json!({"event":"route","decision":if all_routes_enabled {"allow"} else {"deny"},"route":value}),
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
    if force_push_requested(argv) {
        return Ok(json!({
            "ok": false,
            "decision": "deny",
            "reason": "force-push is a non-toggleable hard safety boundary",
            "mutation": true,
            "redacted_preview": command_preview(argv),
        }));
    }
    if mutation && !session_capability_enabled("github_mutation") {
        return Ok(json!({
            "ok": false,
            "decision": "deny",
            "reason": "GitHub mutation is disabled by the optional session routing switch",
            "mutation": true,
            "redacted_preview": command_preview(argv),
        }));
    }
    if (first == "gh" || git_network(argv)) && !session_capability_enabled("network") {
        return Ok(json!({
            "ok": false,
            "decision": "deny",
            "reason": "network is disabled by the optional session routing switch",
            "mutation": mutation,
            "redacted_preview": command_preview(argv),
        }));
    }
    let mut result = json!({
        "ok": true,
        "decision": if mutation {"guarded"} else {"read_only"},
        "mutation": mutation,
        "executed": false,
        "decision_id": decision_id.or_else(|| mutation.then_some(USER_FULL_ACCESS_DECISION_ID)),
        "redacted_preview": command_preview(argv),
    });
    if execute {
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]);
        remove_secret_env(&mut command);
        let status = command.status()?;
        result["executed"] = json!(true);
        result["exit_code"] = json!(status.code().unwrap_or(1));
        result["ok"] = json!(status.success());
        if !status.success() {
            result["decision"] = json!("command_failed");
        }
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

pub fn openrouter_model_catalog_summary(models_json: &Value, target_model: &str) -> Value {
    let models = models_json
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let model_count = models.len();
    let has_openai = models.iter().any(|m| {
        m.get("id")
            .and_then(Value::as_str)
            .map(|id| id.starts_with("openai/"))
            .unwrap_or(false)
    });
    let has_anthropic = models.iter().any(|m| {
        m.get("id")
            .and_then(Value::as_str)
            .map(|id| id.starts_with("anthropic/"))
            .unwrap_or(false)
    });
    let has_target_model = models.iter().any(|m| {
        m.get("id")
            .and_then(Value::as_str)
            .map(|id| id == target_model)
            .unwrap_or(false)
    });

    json!({
        "model_count": model_count,
        "has_openai_models": has_openai,
        "has_anthropic_models": has_anthropic,
        "target_model": target_model,
        "has_target_model": has_target_model,
    })
}

pub fn openrouter_wire_compatibility_summary(openapi_json: &Value) -> Value {
    fn has_post_path(openapi_json: &Value, path: &str) -> bool {
        openapi_json
            .get("paths")
            .and_then(|paths| paths.get(path))
            .and_then(|path_item| path_item.get("post"))
            .is_some()
    }

    let responses_api_documented = has_post_path(openapi_json, "/responses");
    let chat_completions_documented = has_post_path(openapi_json, "/chat/completions");

    json!({
        "source": "https://openrouter.ai/openapi.json",
        "responses_api_documented": responses_api_documented,
        "chat_completions_documented": chat_completions_documented,
        "direct_responses_wire_compatible": responses_api_documented,
        "chat_completion_fallback_available": chat_completions_documented,
    })
}

pub fn openrouter_probe_value(model: Option<&str>, prompt: Option<&str>) -> Result<Value> {
    if !session_capability_enabled("external_providers") || !session_capability_enabled("network") {
        return Err(anyhow!(
            "OpenRouter is disabled by optional external-provider or network routing switches"
        ));
    }
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
    let catalog_summary = openrouter_model_catalog_summary(&models_json, model);
    let model_count = catalog_summary
        .get("model_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let has_openai = catalog_summary
        .get("has_openai_models")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let has_anthropic = catalog_summary
        .get("has_anthropic_models")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut openapi_cmd = Command::new("curl");
    openapi_cmd.args([
        "-sS",
        "-L",
        "-A",
        "codex-harness-openrouter-proof/1.0",
        "https://openrouter.ai/openapi.json",
    ]);
    let (openapi_exit, openapi_stdout, openapi_stderr) = command_output_text(openapi_cmd)?;
    let openapi_json: Value = serde_json::from_str(&openapi_stdout).unwrap_or_else(
        |_| json!({"parse_error": redact(&openapi_stderr), "raw_len": openapi_stdout.len()}),
    );
    let mut wire_compatibility = openrouter_wire_compatibility_summary(&openapi_json);
    wire_compatibility["openapi_exit_code"] = json!(openapi_exit);
    wire_compatibility["openapi_parse_ok"] = json!(openapi_json.get("paths").is_some());
    let direct_responses_wire_compatible = wire_compatibility
        .get("direct_responses_wire_compatible")
        .and_then(Value::as_bool)
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
    let catalog_ok = models_exit == 0
        && model_count > 0
        && has_openai
        && has_anthropic
        && direct_responses_wire_compatible;
    let execution_ready = catalog_ok && key_valid && generation_ok;
    let provider_contract_fingerprint = provider_contract_fingerprint("openrouter")?;
    let provider_receipt_ts_utc = utc_now();
    let out = json!({
        "ok": execution_ready,
        "catalog_ok": catalog_ok,
        "execution_ready": execution_ready,
        "enabled": true,
        "decision_id": USER_FULL_ACCESS_DECISION_ID,
        "provider": "openrouter",
        "provider_contract_fingerprint": provider_contract_fingerprint,
        "provider_receipt_ts_utc": provider_receipt_ts_utc,
        "base_url": "https://openrouter.ai/api/v1",
        "responses_url": "https://openrouter.ai/api/v1/responses",
        "models_exit_code": models_exit,
        "model_count": model_count,
        "has_openai_models": has_openai,
        "has_anthropic_models": has_anthropic,
        "target_model": model,
        "catalog_summary": catalog_summary,
        "wire_compatibility": wire_compatibility,
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
        json!({
            "event": "openrouter_probe",
            "decision": if execution_ready {"allow"} else {"deny"},
            "provider": "openrouter",
            "provider_contract_fingerprint": out.get("provider_contract_fingerprint"),
            "provider_receipt_ts_utc": out.get("provider_receipt_ts_utc"),
            "result": {
                "ok": execution_ready,
                "executed": has_key,
                "exit_code": if execution_ready {0} else {1},
                "proof": out,
            }
        }),
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
    let provider_contract_fingerprint = provider_contract_fingerprint("claude_bridge")?;
    let claude = which("claude");
    let enabled =
        session_capability_enabled("external_providers") && session_capability_enabled("network");
    let version = claude.as_ref().and_then(|_| {
        let mut cmd = Command::new("claude");
        cmd.arg("--version");
        command_output_text(cmd)
            .ok()
            .map(|(_, stdout, stderr)| redact(&(stdout + &stderr)).trim().to_string())
    });
    let mut result = json!({
        "ok": claude.is_some() && enabled,
        "enabled": enabled,
        "decision_id": if enabled {Value::String(USER_FULL_ACCESS_DECISION_ID.to_string())} else {Value::Null},
        "claude_path": claude.map(|p| p.display().to_string()),
        "version": version,
        "mode": "supervised-external-cli-bridge",
        "safety_mode": "safe-mode-no-tools-strict-empty-mcp",
        "tools": [],
        "mcp_servers": [],
        "slash_commands": false,
        "chrome": false,
        "session_persistence": false,
        "auth_mode": if allow_default_auth {"claude-default-auth-no-env-secrets"} else {"bare-env-only"},
        "secret_printed": false,
        "executed": false
    });
    if execute {
        if !enabled {
            return Err(anyhow!(
                "Claude bridge is disabled by optional external-provider or network routing switches"
            ));
        }
        let prompt = prompt.unwrap_or("Return compact JSON: {\"bridge\":\"ok\"}.");
        let prompt_hash = sha256_bytes(prompt.as_bytes());
        let cmd = claude_run_command(prompt, allow_default_auth);
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
    let provider_receipt_ts_utc = utc_now();
    append_ledger(
        "network.jsonl",
        json!({"event":"claude_bridge","decision": if result.get("ok").and_then(Value::as_bool).unwrap_or(false) {"allow"} else {"deny"},"provider":"claude_bridge","provider_contract_fingerprint":provider_contract_fingerprint,"provider_receipt_ts_utc":provider_receipt_ts_utc,"result":result}),
    )?;
    Ok(result)
}

pub fn browser_computer_value() -> Result<Value> {
    if !session_capability_enabled("browser_computer") {
        return Err(anyhow!(
            "browser/computer use is disabled by the optional session routing switch"
        ));
    }
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
        "enabled": true,
        "decision_id": USER_FULL_ACCESS_DECISION_ID,
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

    fn write_full_policy(root: &Path) {
        let policy = root.join("codex-harness/policy/policy.toml");
        fs::create_dir_all(policy.parent().unwrap()).unwrap();
        fs::write(
            policy,
            format!(
                r#"
full_access_decision_id = "{USER_FULL_ACCESS_DECISION_ID}"

[permission_grants]
operator_grants_are_execution_context = true
expanded_access_is_not_a_blocker = true
decision_id = "{USER_FULL_ACCESS_DECISION_ID}"
danger_full_access = "keep"
"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn policy_denies_rm() {
        let d = policy_decision(&["rm".into(), "-rf".into(), "/tmp/x".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
    }

    #[test]
    fn policy_denies_ledger_write() {
        let d = policy_decision(&[
            "tee".into(),
            "/home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/ledger/harness.jsonl"
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
    fn policy_delegates_exact_codex_full_access_frontdoor_to_live_permissions() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        let d = policy_decision(&[
            "codex".into(),
            "--dangerously-bypass-approvals-and-sandbox".into(),
        ]);
        assert_eq!(d.decision, DecisionKind::Allow);
        assert!(d.reason.contains("/permissions"));
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn arbitrary_danger_text_does_not_bypass_normal_policy() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        let d = policy_decision(&[
            "tool".into(),
            "--danger-full-access".into(),
            "--decision-id".into(),
            "DEC-1".into(),
        ]);
        assert_eq!(d.decision, DecisionKind::Prompt);
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn policy_denies_github_mutation_without_guard() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        let d = policy_decision(&["gh".into(), "pr".into(), "merge".into(), "1".into()]);
        assert_eq!(d.decision, DecisionKind::Deny);
        assert_eq!(
            d.violation.as_deref(),
            Some("github_mutation_without_guard")
        );
        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn danger_text_cannot_bypass_delete_or_force_push_safety() {
        let delete = policy_decision(&[
            "bash".into(),
            "-c".into(),
            "rm -rf /tmp/definitely-not-executed".into(),
            "--danger-full-access".into(),
        ]);
        assert_ne!(delete.decision, DecisionKind::Allow);

        let force = policy_decision(&[
            "bash".into(),
            "-c".into(),
            "git push --force origin main".into(),
            "bypass".into(),
        ]);
        assert_ne!(force.decision, DecisionKind::Allow);
    }

    #[test]
    fn codex_frontdoor_path_rejects_user_local_and_temporary_shadows() {
        assert!(approved_codex_frontdoor_path(Path::new(
            "/home/flexnetos/.nix-profile/bin/codex"
        )));
        assert!(approved_codex_frontdoor_path(Path::new(
            "/nix/store/example/toolbin/codex"
        )));
        assert!(!approved_codex_frontdoor_path(Path::new(
            "/home/flexnetos/.local/bin/codex"
        )));
        assert!(!approved_codex_frontdoor_path(Path::new("/tmp/codex")));
    }

    #[test]
    fn model_router_marker_unlocks_subagent_depth_one() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_PERMISSION_PROFILE", ":danger-full-access");
        write_full_policy(dir.path());
        fs::create_dir_all(dir.path().join("codex-harness/ledger")).unwrap();
        route_model_tasks(&["containment policy tests".to_string()]).unwrap();
        let input = json!({"hook_event_name":"SubagentStart","depth":1});
        let resp = hook_response(&input).unwrap();
        assert!(resp.as_object().map(|o| o.is_empty()).unwrap_or(false));
        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_PERMISSION_PROFILE");
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

    #[test]
    fn tracked_operator_grant_survives_clean_runtime_state() {
        let dir = tempfile::tempdir().unwrap();
        let policy = dir.path().join("policy.toml");
        fs::write(
            &policy,
            format!(
                r#"
full_access_decision_id = "{USER_FULL_ACCESS_DECISION_ID}"

[permission_grants]
operator_grants_are_execution_context = true
expanded_access_is_not_a_blocker = true
decision_id = "{USER_FULL_ACCESS_DECISION_ID}"
danger_full_access = "keep"
"#
            ),
        )
        .unwrap();
        assert!(tracked_full_access_policy_granted_at(&policy));
        assert!(!dir.path().join("full-access-grant.json").exists());
    }

    #[test]
    fn tracked_operator_grant_fails_closed_when_incomplete() {
        let dir = tempfile::tempdir().unwrap();
        let policy = dir.path().join("policy.toml");
        fs::write(
            &policy,
            format!(
                r#"
full_access_decision_id = "{USER_FULL_ACCESS_DECISION_ID}"

[permission_grants]
operator_grants_are_execution_context = true
expanded_access_is_not_a_blocker = false
decision_id = "{USER_FULL_ACCESS_DECISION_ID}"
danger_full_access = "keep"
"#
            ),
        )
        .unwrap();
        assert!(!tracked_full_access_policy_granted_at(&policy));
    }

    #[test]
    fn codex_permission_profile_is_informational_not_a_duplicate_gate() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        write_full_policy(dir.path());

        env::set_var("CODEX_PERMISSION_PROFILE", ":danger-full-access");
        assert!(tracked_full_access_policy_granted());
        assert!(session_capability_enabled("external_providers"));

        env::set_var("CODEX_PERMISSION_PROFILE", "harness-read-only");
        assert!(tracked_full_access_policy_granted());
        assert!(session_capability_enabled("external_providers"));
        assert!(session_capability_enabled("subagents"));

        env::set_var("CODEX_PERMISSION_PROFILE", "harness-local-models");
        assert!(tracked_full_access_policy_granted());
        assert!(session_capability_enabled("local_models"));
        assert!(session_capability_enabled("external_providers"));

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_PERMISSION_PROFILE");
    }

    #[test]
    fn missing_permission_profile_does_not_recreate_full_access_deadlock() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::remove_var("CODEX_PERMISSION_PROFILE");
        write_full_policy(dir.path());

        assert!(tracked_full_access_policy_granted());
        assert!(session_capability_enabled("external_providers"));
        assert!(session_capability_enabled("local_models"));
        assert!(session_capability_enabled("network"));
        assert!(session_capability_enabled("github_mutation"));
        assert!(session_capability_enabled("browser_computer"));
        assert!(session_capability_enabled("background_jobs"));
        assert!(session_capability_enabled("subagents"));

        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn corrupt_session_capability_state_fails_closed() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_THREAD_ID", "corrupt-session-state");
        write_full_policy(dir.path());
        let path = session_capability_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{broken").unwrap();

        for capability in SESSION_CAPABILITIES {
            assert!(!session_capability_enabled(capability), "{capability}");
        }
        let status = session_capability_status();
        assert_eq!(
            status.get("state_valid").and_then(Value::as_bool),
            Some(false)
        );
        assert!(set_session_capability("network", false).is_err());

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_THREAD_ID");
    }

    #[test]
    fn session_toggle_narrows_optional_routing_without_claiming_os_authority() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_THREAD_ID", "test-session-toggle");
        write_full_policy(dir.path());

        env::set_var("CODEX_PERMISSION_PROFILE", ":danger-full-access");
        assert!(session_capability_enabled("github_mutation"));
        set_session_capability("github_mutation", false).unwrap();
        assert!(!session_capability_enabled("github_mutation"));
        set_session_capability("github_mutation", true).unwrap();
        assert!(session_capability_enabled("github_mutation"));

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_PERMISSION_PROFILE");
        env::remove_var("CODEX_THREAD_ID");
    }

    #[test]
    fn local_model_tasks_route_to_ollama() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_PERMISSION_PROFILE", "harness-local-models");
        write_full_policy(dir.path());

        let route = model_route_for_task("summarize with local Ollama");
        assert_eq!(
            route.get("provider").and_then(Value::as_str),
            Some("ollama")
        );
        assert_eq!(
            route.get("profile").and_then(Value::as_str),
            Some("envctl-local-models")
        );
        assert_eq!(
            route
                .get("approved_capability_expansion")
                .and_then(Value::as_bool),
            Some(true)
        );

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_PERMISSION_PROFILE");
    }

    #[test]
    fn openai_task_classes_route_to_sol_terra_and_luna() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        write_full_policy(dir.path());

        for (task, expected) in [
            ("high-stakes security review", "gpt-5.6-sol"),
            ("balanced Terra professional workflow", "gpt-5.6-terra"),
            (
                "simple high-throughput mechanical inventory",
                "gpt-5.6-luna",
            ),
        ] {
            let route = model_route_for_task(task);
            assert_eq!(
                route.get("model").and_then(Value::as_str),
                Some(expected),
                "{task}"
            );
        }

        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn explicit_provider_routing_precedes_sol_terra_luna_task_classes() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());

        for (task, provider) in [
            ("simple local Ollama inventory", "ollama"),
            ("high-stakes Claude security review", "claude-bridge"),
            ("simple OpenRouter summary", "openrouter"),
        ] {
            let route = model_route_for_task(task);
            assert_eq!(
                route.get("provider").and_then(Value::as_str),
                Some(provider),
                "{task}"
            );
            assert_eq!(
                route
                    .get("approved_capability_expansion")
                    .and_then(Value::as_bool),
                Some(true),
                "{task}"
            );
        }

        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn network_toggle_blocks_native_codex_routes_and_children() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_THREAD_ID", "network-off-native-codex");
        write_full_policy(dir.path());
        route_model_tasks(&["native Codex implementation".to_string()]).unwrap();
        set_session_capability("network", false).unwrap();

        let route = model_route_for_task("native Codex implementation");
        assert_eq!(
            route
                .get("approved_capability_expansion")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert!(spawn_codex_exec(dir.path(), "envctl-harness", "test").is_err());
        assert!(run_codex_exec(dir.path(), "envctl-harness", "test").is_err());

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_THREAD_ID");
    }

    #[test]
    fn network_toggle_blocks_remote_git_operations_and_push() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_THREAD_ID", "network-off-git");
        write_full_policy(dir.path());
        set_session_capability("network", false).unwrap();

        for argv in [
            vec!["git".into(), "push".into(), "origin".into(), "main".into()],
            vec!["git".into(), "fetch".into(), "origin".into()],
            vec!["git".into(), "pull".into(), "--ff-only".into()],
            vec![
                "git".into(),
                "clone".into(),
                "https://example.invalid/x".into(),
            ],
            vec!["git".into(), "ls-remote".into(), "origin".into()],
            vec!["git".into(), "remote".into(), "update".into()],
            vec!["git".into(), "submodule".into(), "update".into()],
        ] {
            let result = github_guard_check(&argv, Some("operator"), false).unwrap();
            assert_eq!(result.get("ok").and_then(Value::as_bool), Some(false));
            assert_eq!(
                result.get("reason").and_then(Value::as_str),
                Some("network is disabled by the optional session routing switch")
            );
        }

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_THREAD_ID");
    }

    #[test]
    fn subagent_start_rechecks_current_network_toggle() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_THREAD_ID", "subagent-network-recheck");
        write_full_policy(dir.path());
        route_model_tasks(&["native Codex implementation".to_string()]).unwrap();
        set_session_capability("network", false).unwrap();

        let response =
            hook_response(&json!({"hook_event_name":"SubagentStart","depth":1})).unwrap();
        assert_eq!(
            response
                .pointer("/hookSpecificOutput/permissionDecision")
                .and_then(Value::as_str),
            Some("deny")
        );
        assert!(response
            .pointer("/hookSpecificOutput/permissionDecisionReason")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .contains("network"));

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_THREAD_ID");
    }

    #[test]
    fn github_guard_never_allows_force_push() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        write_full_policy(dir.path());

        for argv in [
            vec!["git".into(), "push".into(), "--force".into()],
            vec!["git".into(), "push".into(), "--force-with-lease".into()],
            vec![
                "git".into(),
                "push".into(),
                "--force-with-lease=refs/heads/main:deadbeef".into(),
            ],
            vec!["git".into(), "push".into(), "-f".into()],
            vec![
                "git".into(),
                "push".into(),
                "origin".into(),
                "+refs/heads/main:refs/heads/main".into(),
            ],
            vec!["git".into(), "push".into(), "--mirror".into()],
        ] {
            let result = github_guard_check(&argv, Some("operator"), false).unwrap();
            assert_eq!(result.get("ok").and_then(Value::as_bool), Some(false));
            assert_eq!(
                result.get("reason").and_then(Value::as_str),
                Some("force-push is a non-toggleable hard safety boundary")
            );
        }

        env::remove_var("CODEX_HARNESS_ROOT");
    }

    #[test]
    fn github_guard_reuses_enabled_session_intent_without_extra_id_gate() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        env::set_var("CODEX_THREAD_ID", "github-default-decision");

        let result = github_guard_check(
            &["gh".into(), "pr".into(), "merge".into(), "123".into()],
            None,
            false,
        )
        .unwrap();
        assert_eq!(result.get("ok").and_then(Value::as_bool), Some(true));
        assert_eq!(
            result.get("decision_id").and_then(Value::as_str),
            Some(USER_FULL_ACCESS_DECISION_ID)
        );

        env::remove_var("CODEX_HARNESS_ROOT");
        env::remove_var("CODEX_THREAD_ID");
    }

    #[test]
    fn claude_child_has_no_tools_or_customization_escape_hatches() {
        let bare = claude_run_argv("test prompt", false);
        assert_eq!(bare.first().map(String::as_str), Some("claude"));
        assert!(bare.iter().any(|arg| arg == "--bare"));
        assert!(bare.iter().any(|arg| arg == "--safe-mode"));
        assert!(bare.windows(2).any(|pair| pair == ["--tools", ""]));
        assert!(bare
            .windows(2)
            .any(|pair| pair == ["--mcp-config", r#"{"mcpServers":{}}"#]));
        assert!(bare.iter().any(|arg| arg == "--strict-mcp-config"));
        assert!(bare.iter().any(|arg| arg == "--disable-slash-commands"));
        assert!(bare.iter().any(|arg| arg == "--no-chrome"));
        assert!(bare.iter().any(|arg| arg == "--no-session-persistence"));
        assert!(!bare
            .iter()
            .any(|arg| arg == "--dangerously-skip-permissions"));
        assert!(!bare
            .iter()
            .any(|arg| arg == "bypassPermissions" || arg == "--permission-mode"));
        for forbidden_tool in ["Read", "Bash", "Edit", "Write", "Agent"] {
            assert!(!bare.iter().any(|arg| arg == forbidden_tool));
        }

        let default_auth = claude_run_argv("test prompt", true);
        assert!(!default_auth.iter().any(|arg| arg == "--bare"));
        assert!(default_auth.windows(2).any(|pair| pair == ["--tools", ""]));

        let command = claude_run_command("test prompt", true);
        for secret in [
            "OPENAI_API_KEY",
            "CODEX_API_KEY",
            "OPENROUTER_API_KEY",
            "ANTHROPIC_API_KEY",
            "CLAUDE_API_KEY",
            "GITHUB_TOKEN",
            "GH_TOKEN",
        ] {
            assert!(
                matches!(
                    command
                        .get_envs()
                        .find(|(key, _)| *key == OsStr::new(secret)),
                    Some((_, None))
                ),
                "{secret} must be removed from the Claude child environment"
            );
        }
    }

    #[test]
    fn provider_receipts_use_latest_success_fingerprint_and_freshness() {
        let now = parse_utc_timestamp("2026-07-10T23:30:00Z").unwrap();
        let fingerprint = "current-contract";
        let success = json!({
            "event": "claude_bridge",
            "decision": "allow",
            "provider": "claude_bridge",
            "provider_contract_fingerprint": fingerprint,
            "provider_receipt_ts_utc": "2026-07-10T23:29:00Z",
            "result": {"ok": true, "executed": true, "exit_code": 0},
        });
        assert!(provider_receipt_valid_at(
            std::slice::from_ref(&success),
            "claude_bridge",
            "claude_bridge",
            true,
            fingerprint,
            now,
            PROVIDER_RECEIPT_MAX_AGE,
        ));

        let later_inventory = json!({
            "event": "claude_bridge",
            "decision": "allow",
            "provider": "claude_bridge",
            "provider_contract_fingerprint": fingerprint,
            "provider_receipt_ts_utc": "2026-07-10T23:29:30Z",
            "result": {"ok": true, "executed": false},
        });
        assert!(provider_receipt_valid_at(
            &[success.clone(), later_inventory],
            "claude_bridge",
            "claude_bridge",
            true,
            fingerprint,
            now,
            PROVIDER_RECEIPT_MAX_AGE,
        ));

        let stale = json!({
            "event": "claude_bridge",
            "decision": "allow",
            "provider": "claude_bridge",
            "provider_contract_fingerprint": fingerprint,
            "provider_receipt_ts_utc": "2026-07-10T23:14:59Z",
            "result": {"ok": true, "executed": true, "exit_code": 0},
        });
        assert!(!provider_receipt_valid_at(
            &[stale],
            "claude_bridge",
            "claude_bridge",
            true,
            fingerprint,
            now,
            PROVIDER_RECEIPT_MAX_AGE,
        ));

        let old_ignored_receipt = json!({
            "event": "claude_bridge",
            "decision": "allow",
            "provider_receipt_ts_utc": "2026-07-10T23:29:00Z",
            "result": {"ok": true, "executed": true, "exit_code": 0},
        });
        assert!(!provider_receipt_valid_at(
            &[old_ignored_receipt],
            "claude_bridge",
            "claude_bridge",
            true,
            fingerprint,
            now,
            PROVIDER_RECEIPT_MAX_AGE,
        ));

        let latest_failure = json!({
            "event": "claude_bridge",
            "decision": "deny",
            "provider": "claude_bridge",
            "provider_contract_fingerprint": fingerprint,
            "provider_receipt_ts_utc": "2026-07-10T23:29:30Z",
            "result": {"ok": false, "executed": true, "exit_code": 1},
        });
        assert!(!provider_receipt_valid_at(
            &[success, latest_failure],
            "claude_bridge",
            "claude_bridge",
            true,
            fingerprint,
            now,
            PROVIDER_RECEIPT_MAX_AGE,
        ));
    }

    #[test]
    fn provider_contract_fingerprint_is_provider_specific_and_source_bound() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        env::set_var("CODEX_HARNESS_ROOT", dir.path());
        let harness = dir.path().join("codex-harness");
        fs::create_dir_all(harness.join("src")).unwrap();
        fs::create_dir_all(harness.join("config/policy")).unwrap();
        fs::write(harness.join("src/lib.rs"), COMPILED_PROVIDER_SOURCE).unwrap();
        fs::write(
            harness.join("config/policy/providers.toml"),
            COMPILED_PROVIDER_CONFIG,
        )
        .unwrap();

        let claude = provider_contract_fingerprint("claude_bridge").unwrap();
        let ollama = provider_contract_fingerprint("ollama").unwrap();
        assert_ne!(claude, ollama);
        fs::write(harness.join("config/policy/providers.toml"), b"changed").unwrap();
        assert!(provider_contract_fingerprint("claude_bridge").is_err());

        env::remove_var("CODEX_HARNESS_ROOT");
    }
}
