//! The agent-asset lock — SHA-256 content lock for provisioned skills/MCPs/commands.
//! Consolidates kasetto v3.2.0 `src/lock.rs` + the mode/drift logic in
//! `src/commands/lock.rs`.
//!
//! This is a **separate type** from the engine's FNV-1a component lock
//! (`crates/engine/src/lock.rs`); the two never share code. TASK-0016 keeps the
//! SHA-256 agent-asset lock as the standalone committed `agent-env.lock` because
//! folding it into the component lock would mix two hash domains and weaken the
//! no-downgrade boundary.
//!
//! ## 3 modes ([`LockMode`])
//! - [`LockMode::Plain`] — verify + fetch as needed, write/refresh the lock.
//! - [`LockMode::Update`] — re-resolve the named packages' refs and rewrite the lock.
//! - [`LockMode::Locked`] — verify the lock is satisfied with **zero network fetch**;
//!   fail-closed if it isn't.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::fs;

use serde::{Deserialize, Serialize};

use crate::config::Scope;
use crate::{err, Result};

/// Schema version of the agent-asset lock section.
///
/// Version 3 introduces the `tree-v1` skill hash domain: effective entry kind, native path bytes,
/// permission mode, and file bytes are length-framed into the digest. Project-scope snapshots are
/// first made Git-portable (canonical 0755 directories, 0644/0755 files, and no uncommittable
/// empty directories), while machine-local global snapshots retain their exact effective modes.
/// A v2 skill hash is therefore intentionally not reinterpreted as v3.
pub const LOCK_VERSION: u8 = 3;

/// Refuse a lock written by a newer envctl before its fields can be discarded or restamped.
/// Older schemas remain readable because plain sync owns their full, evidence-aware migration.
pub(crate) fn ensure_supported_version(version: u8) -> Result<()> {
    if version > LOCK_VERSION {
        return Err(err(format!(
            "agent lock schema version {version} is newer than supported version {LOCK_VERSION}; refusing to migrate it backward"
        )));
    }
    Ok(())
}

/// Reserved top-level key for exporters that need to label this lock domain.
/// The committed envctl repo keeps this data in standalone `agent-env.lock`.
pub const AGENT_ASSETS_KEY: &str = "agent_assets";

/// Default standalone filename for the agent-asset lock.
pub const LOCK_FILENAME: &str = "agent-env.lock";

/// A tracked skill entry (the verbatim-copied skill tree, SHA-256 hashed).
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct AgentLockEntry {
    /// Install path relative to the scope root (portable across machines); legacy locks
    /// may store an absolute path here, which is still honored.
    pub destination: String,
    pub hash: String,
    pub skill: String,
    #[serde(default)]
    pub description: String,
    pub source: String,
    pub source_revision: String,
    /// Scope this entry was installed under (present for locks written by newer envctl).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
}

/// A tracked non-skill asset (command or MCP) recorded in the lock.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct AssetEntry {
    pub kind: String,
    pub name: String,
    pub hash: String,
    pub source: String,
    /// For commands: install paths relative to the scope root.
    /// For MCPs: merged server names. Version 3 stores both as an injective,
    /// length-framed list; legacy comma-separated values are read only for v2 migration.
    pub destination: String,
    /// Resolved git revision label (e.g. `ref:v1.0`, `branch:main`, `local`). Defaulted to
    /// empty for backwards compatibility with v2 locks written before this field existed;
    /// drift checks skip the revision comparison when this is empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_revision: String,
}

/// Durable proof that a successful project-scope apply installed one exact output unit.
///
/// This is deliberately separate from desired content entries and selectors: `agent lock`
/// preserves existing claims but never fabricates new ownership.  Paths are relative to the
/// project scope root, so a checked-in lock remains valid in a fresh checkout at a new path.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct InstalledOutput {
    pub asset_id: String,
    pub destination: String,
    pub format: String,
    pub unit: String,
    pub hash: String,
}

/// Portable, commit-friendly manifest of installed agent assets (skills + commands/MCPs).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AgentLockFile {
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default)]
    pub skills: BTreeMap<String, AgentLockEntry>,
    #[serde(default)]
    pub assets: BTreeMap<String, AssetEntry>,
    /// Canonical selectors for lock entries, keyed by the corresponding skill/asset id.
    /// A content hash is not self-authenticating when `sub-dir`, wildcard/list mode, or an
    /// object `path` can change which bytes the same source/revision/name selects. Older v2
    /// locks omit this map; a zero-network audit reports that omission as drift instead of
    /// guessing, for local and remote sources alike.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub source_selectors: BTreeMap<String, String>,
    /// Portable project-scope ownership attestations written only after successful apply.
    /// Removed assets/targets may retain claims as tombstones until a later plain sync safely
    /// removes the exact output and compacts the map.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub installed_outputs: BTreeMap<String, InstalledOutput>,
}

fn default_version() -> u8 {
    // A versionless file predates the explicit tree-v1 domain. Never let serde omission claim
    // current semantics; callers may inspect it for migration, but sync/save fail closed.
    2
}

impl Default for AgentLockFile {
    fn default() -> Self {
        Self {
            version: LOCK_VERSION,
            skills: BTreeMap::new(),
            assets: BTreeMap::new(),
            source_selectors: BTreeMap::new(),
            installed_outputs: BTreeMap::new(),
        }
    }
}

/// The mode a lock operation runs in (kasetto's `sync`/`lock` mode logic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockMode {
    /// Verify + fetch as needed, write/refresh the lock (default).
    Plain,
    /// Re-resolve the named packages' refs and rewrite the lock (`--update <name>...`);
    /// empty means re-resolve every source.
    Update(Vec<String>),
    /// Verify the lock is satisfied with ZERO network fetch; fail-closed if unsatisfied.
    Locked,
}

impl LockMode {
    /// Whether this mode permits any network fetch. `Locked` is the only zero-network mode.
    pub fn allows_fetch(&self) -> bool {
        !matches!(self, LockMode::Locked)
    }

    /// Whether a source named `source_url` providing `skill` should be re-resolved under
    /// this mode. `Plain` re-resolves all; `Update(names)` only sources whose tracked
    /// skills intersect `names`; `Locked` never re-resolves.
    pub fn should_resolve(&self, source_url: &str, prev: &AgentLockFile) -> bool {
        match self {
            LockMode::Plain => true,
            LockMode::Locked => false,
            LockMode::Update(names) => {
                if names.is_empty() {
                    return true;
                }
                prev.skills
                    .values()
                    .any(|e| e.source == source_url && names.contains(&e.skill))
            }
        }
    }
}

/// A portable snapshot of the skill subset of a lock, used for state transfer
/// (kasetto `LockFile::state` / `apply_state`).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AgentLockState {
    pub version: u8,
    pub skills: BTreeMap<String, AgentLockEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub source_selectors: BTreeMap<String, String>,
}

impl Default for AgentLockState {
    fn default() -> Self {
        Self {
            version: LOCK_VERSION,
            skills: BTreeMap::new(),
            source_selectors: BTreeMap::new(),
        }
    }
}

/// One drift change between two lock snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockDrift {
    pub status: DriftStatus,
    pub id: String,
}

/// The kind of drift for a single lock entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftStatus {
    Added,
    Removed,
    Updated,
}

impl DriftStatus {
    pub fn label(self) -> &'static str {
        match self {
            DriftStatus::Added => "added",
            DriftStatus::Removed => "removed",
            DriftStatus::Updated => "updated",
        }
    }
}

impl AgentLockFile {
    pub fn get_tracked_asset(&self, kind: &str, id: &str) -> Option<(String, String)> {
        self.assets.get(id).and_then(|a| {
            if a.kind == kind {
                Some((a.hash.clone(), a.destination.clone()))
            } else {
                None
            }
        })
    }

    pub fn save_tracked_asset(&mut self, id: &str, entry: AssetEntry) {
        self.assets.insert(id.to_string(), entry);
    }

    pub fn remove_tracked_asset(&mut self, id: &str) {
        self.assets.remove(id);
        self.source_selectors.remove(id);
    }

    pub fn set_source_selector(&mut self, id: &str, selector: Option<String>) {
        match selector {
            Some(selector) => {
                self.source_selectors.insert(id.to_string(), selector);
            }
            None => {
                self.source_selectors.remove(id);
            }
        }
    }

    pub fn list_tracked_asset_ids(&self, kind: &str) -> Vec<(&str, &str)> {
        self.assets
            .iter()
            .filter(|(_, a)| a.kind == kind)
            .map(|(id, a)| (id.as_str(), a.destination.as_str()))
            .collect()
    }

    pub fn clear_all(&mut self) {
        self.skills.clear();
        self.assets.clear();
        self.source_selectors.clear();
        self.installed_outputs.clear();
    }

    /// Capture a portable snapshot of the skill state (kasetto `LockFile::state`).
    pub fn state(&self) -> AgentLockState {
        AgentLockState {
            version: self.version,
            skills: self.skills.clone(),
            source_selectors: self
                .source_selectors
                .iter()
                .filter(|(id, _)| self.skills.contains_key(*id))
                .map(|(id, selector)| (id.clone(), selector.clone()))
                .collect(),
        }
    }

    /// Restore the skill state from a snapshot (kasetto `LockFile::apply_state`).
    pub fn apply_state(&mut self, state: &AgentLockState) {
        self.version = state.version;
        self.source_selectors
            .retain(|id, _| !self.skills.contains_key(id));
        self.skills = state.skills.clone();
        self.source_selectors.extend(state.source_selectors.clone());
    }

    /// Sorted, deduplicated list of installed command names from tracked assets.
    pub fn list_installed_commands(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .assets
            .iter()
            .filter(|(_, a)| a.kind == "command")
            .map(|(_, a)| a.name.clone())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// Sorted, deduplicated list of installed MCP server names from tracked assets.
    pub fn list_installed_mcps(&self) -> Result<Vec<String>> {
        let mut servers = Vec::new();
        for (_, destinations) in self.list_tracked_asset_ids("mcp") {
            servers.extend(decode_asset_list(destinations, self.version)?);
        }
        servers.sort();
        servers.dedup();
        Ok(servers)
    }

    /// Compute drift between this (previous on-disk) lock and a freshly-resolved `next` lock.
    /// Deterministic order via BTreeMap iteration. The basis of `lock --check` (CI drift,
    /// no mutation): a non-empty result means the on-disk lock is out of date.
    pub fn lock_check(&self, next: &AgentLockFile) -> Vec<LockDrift> {
        let mut out = Vec::new();
        if self.version != next.version {
            out.push(LockDrift {
                status: DriftStatus::Updated,
                id: "version".into(),
            });
        }
        for (id, prev) in &self.skills {
            match next.skills.get(id) {
                None => out.push(LockDrift {
                    status: DriftStatus::Removed,
                    id: id.clone(),
                }),
                Some(now) if now != prev => {
                    out.push(LockDrift {
                        status: DriftStatus::Updated,
                        id: id.clone(),
                    });
                }
                _ => {}
            }
        }
        for id in next.skills.keys() {
            if !self.skills.contains_key(id) {
                out.push(LockDrift {
                    status: DriftStatus::Added,
                    id: id.clone(),
                });
            }
        }
        for (id, prev) in &self.assets {
            match next.assets.get(id) {
                None => out.push(LockDrift {
                    status: DriftStatus::Removed,
                    id: id.clone(),
                }),
                Some(now) if now != prev => {
                    out.push(LockDrift {
                        status: DriftStatus::Updated,
                        id: id.clone(),
                    });
                }
                _ => {}
            }
        }
        for id in next.assets.keys() {
            if !self.assets.contains_key(id) {
                out.push(LockDrift {
                    status: DriftStatus::Added,
                    id: id.clone(),
                });
            }
        }
        for (id, previous) in &self.source_selectors {
            match next.source_selectors.get(id) {
                None => out.push(LockDrift {
                    status: DriftStatus::Removed,
                    id: format!("selector::{id}"),
                }),
                Some(current) if current != previous => out.push(LockDrift {
                    status: DriftStatus::Updated,
                    id: format!("selector::{id}"),
                }),
                _ => {}
            }
        }
        for id in next.source_selectors.keys() {
            if !self.source_selectors.contains_key(id) {
                out.push(LockDrift {
                    status: DriftStatus::Added,
                    id: format!("selector::{id}"),
                });
            }
        }
        for (id, previous) in &self.installed_outputs {
            match next.installed_outputs.get(id) {
                None => out.push(LockDrift {
                    status: DriftStatus::Removed,
                    id: format!("installed-output::{id}"),
                }),
                Some(current) if current != previous => out.push(LockDrift {
                    status: DriftStatus::Updated,
                    id: format!("installed-output::{id}"),
                }),
                _ => {}
            }
        }
        for id in next.installed_outputs.keys() {
            if !self.installed_outputs.contains_key(id) {
                out.push(LockDrift {
                    status: DriftStatus::Added,
                    id: format!("installed-output::{id}"),
                });
            }
        }
        out
    }
}

/// Injective key for one installed-output ownership unit.
pub fn installed_output_key(asset_id: &str, destination: &str, format: &str, unit: &str) -> String {
    format!(
        "{}:{asset_id}|{}:{destination}|{}:{format}|{}:{unit}",
        asset_id.len(),
        destination.len(),
        format.len(),
        unit.len()
    )
}

/// Encode a v3 list without delimiter ambiguity (paths and server names may contain commas).
pub fn encode_asset_list(items: impl IntoIterator<Item = String>) -> String {
    let items = items.into_iter().collect::<Vec<_>>();
    let mut encoded = format!("v3|{}|", items.len());
    for item in items {
        encoded.push_str(&format!("{}:{item}", item.len()));
    }
    encoded
}

/// Decode a v3 length-framed list. Legacy CSV is accepted only when the containing lock is v2.
pub fn decode_asset_list(value: &str, lock_version: u8) -> Result<Vec<String>> {
    const MAX_LIST_ITEMS: usize = 4096;
    const MAX_LIST_BYTES: usize = 4 * 1024 * 1024;
    if value.len() > MAX_LIST_BYTES {
        return Err(err("asset list exceeds the maximum encoded size"));
    }
    if lock_version < LOCK_VERSION {
        return Ok(value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect());
    }
    let rest = value
        .strip_prefix("v3|")
        .ok_or_else(|| err("v3 asset list is missing the `v3|` framing prefix"))?;
    let (count, mut rest) = rest
        .split_once('|')
        .ok_or_else(|| err("v3 asset list is missing its item count delimiter"))?;
    let count = count
        .parse::<usize>()
        .map_err(|_| err("v3 asset list has an invalid item count"))?;
    if count > MAX_LIST_ITEMS {
        return Err(err("v3 asset list exceeds the maximum item count"));
    }
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let colon = rest
            .find(':')
            .ok_or_else(|| err("v3 asset list item is missing its length delimiter"))?;
        let len = rest[..colon]
            .parse::<usize>()
            .map_err(|_| err("v3 asset list item has an invalid length"))?;
        let bytes = &rest.as_bytes()[colon + 1..];
        if bytes.len() < len || !rest[colon + 1..].is_char_boundary(len) {
            return Err(err("v3 asset list item length is out of bounds"));
        }
        let (item, tail) = rest[colon + 1..].split_at(len);
        items.push(item.to_string());
        rest = tail;
    }
    if !rest.is_empty() {
        return Err(err("v3 asset list has trailing data"));
    }
    Ok(items)
}

/// Resolve the standalone lock file path for the given scope.
/// `Project` → `<project_root>/<LOCK_FILENAME>`, `Global` → `<global_data_dir>/<LOCK_FILENAME>`.
pub fn lock_path(scope: Scope, project_root: &Path, global_data_dir: &Path) -> PathBuf {
    match scope {
        Scope::Project => project_root.join(LOCK_FILENAME),
        Scope::Global => global_data_dir.join(LOCK_FILENAME),
    }
}

/// Inspect the lock file from `path` without accepting its schema for mutation.
///
/// This is the read-only diagnostic path used by strict audits that must return typed evidence
/// for an unsupported schema. Mutation paths must use [`load`], which rejects newer schemas
/// before any fields can be discarded or rewritten.
pub fn inspect(path: &Path) -> Result<AgentLockFile> {
    let Some(bytes) = crate::secure_file::read_optional(path)? else {
        return Ok(AgentLockFile::default());
    };
    let text = String::from_utf8(bytes)
        .map_err(|e| err(format!("lock file {} is not UTF-8: {e}", path.display())))?;
    if text.trim().is_empty() {
        return Ok(AgentLockFile::default());
    }
    let lock: AgentLockFile = serde_yaml::from_str(&text)
        .map_err(|e| err(format!("failed to parse lock file {}: {e}", path.display())))?;
    Ok(lock)
}

/// Load the lock file from `path` (or return a default empty one if missing / empty).
pub fn load(path: &Path) -> Result<AgentLockFile> {
    let lock = inspect(path)?;
    ensure_supported_version(lock.version)?;
    Ok(lock)
}

/// Write a current-version lock file to `path`, creating parent directories if needed.
///
/// Older locks must be fully rebuilt before saving; merely relabelling their ambiguous hash bytes
/// would silently reinterpret the v2 hash algorithm as v3.
pub fn save(lock: &mut AgentLockFile, path: &Path) -> Result<()> {
    save_with_mode(lock, path, 0o644)
}

/// Save a lock using the scope's deterministic confidentiality mode.
pub fn save_for_scope(lock: &mut AgentLockFile, path: &Path, scope: Scope) -> Result<()> {
    save_with_mode(
        lock,
        path,
        if scope == Scope::Global { 0o600 } else { 0o644 },
    )
}

fn save_with_mode(lock: &mut AgentLockFile, path: &Path, mode: u32) -> Result<()> {
    if lock.version != LOCK_VERSION {
        return Err(err(format!(
            "refusing to save agent lock version {} as version {LOCK_VERSION}; fully rebuild the lock to migrate tree hashes",
            lock.version
        )));
    }
    let yaml = serde_yaml::to_string(lock)
        .map_err(|e| err(format!("failed to serialize lock file: {e}")))?;
    crate::secure_file::write_atomic(path, yaml.as_bytes(), mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn test_asset(kind: &str, name: &str, destination: &str) -> AssetEntry {
        AssetEntry {
            kind: kind.into(),
            name: name.into(),
            hash: "h".into(),
            source: "s".into(),
            destination: destination.into(),
            source_revision: "rev".into(),
        }
    }

    fn skill_entry(dest: &str, hash: &str, rev: &str) -> AgentLockEntry {
        AgentLockEntry {
            destination: dest.into(),
            hash: hash.into(),
            skill: "skill-a".into(),
            description: "desc".into(),
            source: "src".into(),
            source_revision: rev.into(),
            scope: Some(Scope::Project),
        }
    }

    #[test]
    fn round_trip_with_skills_and_assets() {
        let dir = temp_dir("agent-env-lock-data");
        let path = lock_path(Scope::Project, &dir, &dir);

        let mut lock = AgentLockFile::default();
        lock.skills.insert(
            "src::skill-a".into(),
            skill_entry(".claude/skills/skill-a", "abc", "rev1"),
        );
        lock.save_tracked_asset(
            "mcp::src::pack.json",
            AssetEntry {
                kind: "mcp".into(),
                name: "pack.json".into(),
                hash: "h1".into(),
                source: "src".into(),
                destination: "srv1,srv2".into(),
                source_revision: "rev1".into(),
            },
        );

        save(&mut lock, &path).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded.version, LOCK_VERSION);
        assert_eq!(loaded.skills.len(), 1);
        assert_eq!(loaded.skills["src::skill-a"].hash, "abc");
        // scope-relative destination round-trips verbatim
        assert_eq!(
            loaded.skills["src::skill-a"].destination,
            ".claude/skills/skill-a"
        );
        assert_eq!(
            loaded.get_tracked_asset("mcp", "mcp::src::pack.json"),
            Some(("h1".into(), "srv1,srv2".into()))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_returns_default_when_missing() {
        let dir = temp_dir("agent-env-lock-missing");
        let lock = load(&dir.join("nope.lock")).unwrap();
        assert_eq!(lock.version, LOCK_VERSION);
        assert!(lock.skills.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_lock_loads_for_audit_but_cannot_be_silently_restamped() {
        let dir = temp_dir("agent-env-lock-legacy");
        let path = dir.join(LOCK_FILENAME);

        // A v1 lock carrying fields that no longer exist plus an absolute destination.
        // Unknown fields remain readable for drift/migration diagnostics, but the old hash domain
        // must never be relabelled by save.
        let legacy = "version: 1\n\
last_run: '111'\n\
skills:\n\
\x20 src::a:\n\
\x20\x20\x20 destination: /abs/path/.claude/skills/a\n\
\x20\x20\x20 hash: h\n\
\x20\x20\x20 skill: a\n\
\x20\x20\x20 source: src\n\
\x20\x20\x20 source_revision: local\n\
\x20\x20\x20 updated_at: '111'\n\
assets: {}\n";
        fs::write(&path, legacy).unwrap();

        let mut loaded = load(&path).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.skills.len(), 1);
        assert_eq!(
            loaded.skills["src::a"].destination,
            "/abs/path/.claude/skills/a"
        );

        let before = fs::read_to_string(&path).unwrap();
        let message = save(&mut loaded, &path).unwrap_err().to_string();
        assert!(message.contains("fully rebuild the lock"), "{message}");
        assert_eq!(fs::read_to_string(&path).unwrap(), before);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn versionless_lock_is_legacy_not_implicitly_tree_v1() {
        let lock: AgentLockFile = serde_yaml::from_str(
            "skills: {}\nassets:\n  mcp::remote::pack.json:\n    kind: mcp\n    name: pack.json\n    hash: legacy\n    source: https://example.invalid/repo.git\n    destination: demo\n",
        )
        .expect("parse versionless lock");
        assert_eq!(lock.version, 2);
        let dir = temp_dir("agent-env-versionless-lock");
        let path = dir.join(LOCK_FILENAME);
        let mut lock = lock;
        let message = save(&mut lock, &path).unwrap_err().to_string();
        assert!(message.contains("fully rebuild the lock"), "{message}");
        assert!(!path.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn future_lock_load_is_rejected_without_rewriting_unknown_fields() {
        let dir = temp_dir("agent-env-lock-future");
        let path = dir.join(LOCK_FILENAME);
        let bytes = format!(
            "version: {}\nskills: {{}}\nassets: {{}}\nfuture_schema_field: preserve-verbatim\n",
            LOCK_VERSION + 1
        );
        fs::write(&path, bytes.as_bytes()).unwrap();

        let message = load(&path).unwrap_err().to_string();
        assert!(message.contains("newer than supported"), "{message}");
        assert_eq!(fs::read(&path).unwrap(), bytes.as_bytes());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lock_check_reports_added_removed_updated() {
        let mut prev = AgentLockFile::default();
        prev.skills
            .insert("k::keep".into(), skill_entry("d", "h1", "r1"));
        prev.skills
            .insert("k::gone".into(), skill_entry("d", "h2", "r1"));

        let mut next = AgentLockFile::default();
        // keep unchanged
        next.skills
            .insert("k::keep".into(), skill_entry("d", "h1", "r1"));
        // gone removed; new added
        next.skills
            .insert("k::new".into(), skill_entry("d", "h3", "r1"));

        let drift = prev.lock_check(&next);
        let removed = drift
            .iter()
            .any(|d| d.status == DriftStatus::Removed && d.id == "k::gone");
        let added = drift
            .iter()
            .any(|d| d.status == DriftStatus::Added && d.id == "k::new");
        assert!(removed && added, "drift: {drift:?}");
        // no spurious "keep" drift
        assert!(!drift.iter().any(|d| d.id == "k::keep"));
    }

    #[test]
    fn lock_check_compares_every_skill_and_asset_field() {
        let skill = skill_entry(".claude/skills/skill-a", "abc", "rev1");
        let skill_variants = [
            AgentLockEntry {
                destination: ".codex/skills/skill-a".into(),
                ..skill.clone()
            },
            AgentLockEntry {
                hash: "changed".into(),
                ..skill.clone()
            },
            AgentLockEntry {
                skill: "renamed".into(),
                ..skill.clone()
            },
            AgentLockEntry {
                description: "changed".into(),
                ..skill.clone()
            },
            AgentLockEntry {
                source: "other".into(),
                ..skill.clone()
            },
            AgentLockEntry {
                source_revision: "rev2".into(),
                ..skill.clone()
            },
            AgentLockEntry {
                scope: Some(Scope::Global),
                ..skill.clone()
            },
        ];
        for changed in skill_variants {
            let mut prev = AgentLockFile::default();
            prev.skills.insert("src::skill-a".into(), skill.clone());
            let mut next = prev.clone();
            next.skills.insert("src::skill-a".into(), changed);
            assert_eq!(
                prev.lock_check(&next),
                vec![LockDrift {
                    status: DriftStatus::Updated,
                    id: "src::skill-a".into(),
                }]
            );
        }

        let asset = test_asset("command", "review", ".claude/commands/review.md");
        let asset_variants = [
            AssetEntry {
                kind: "mcp".into(),
                ..asset.clone()
            },
            AssetEntry {
                name: "renamed".into(),
                ..asset.clone()
            },
            AssetEntry {
                hash: "changed".into(),
                ..asset.clone()
            },
            AssetEntry {
                source: "other".into(),
                ..asset.clone()
            },
            AssetEntry {
                destination: ".codex/prompts/review.md".into(),
                ..asset.clone()
            },
            AssetEntry {
                source_revision: "rev2".into(),
                ..asset.clone()
            },
        ];
        for changed in asset_variants {
            let mut prev = AgentLockFile::default();
            prev.save_tracked_asset("command::src::review", asset.clone());
            let mut next = prev.clone();
            next.save_tracked_asset("command::src::review", changed);
            assert_eq!(
                prev.lock_check(&next),
                vec![LockDrift {
                    status: DriftStatus::Updated,
                    id: "command::src::review".into(),
                }]
            );
        }

        let mut prev = AgentLockFile::default();
        prev.save_tracked_asset("command::src::review", asset);
        let mut next = prev.clone();
        next.save_tracked_asset(
            "mcp::src::servers.json",
            test_asset("mcp", "servers.json", "context7"),
        );

        let drift = prev.lock_check(&next);
        assert!(drift
            .iter()
            .any(|d| { d.id == "mcp::src::servers.json" && d.status == DriftStatus::Added }));

        next.assets.remove("command::src::review");
        let drift = prev.lock_check(&next);
        assert!(drift
            .iter()
            .any(|d| { d.id == "command::src::review" && d.status == DriftStatus::Removed }));

        let mut selector_prev = AgentLockFile::default();
        selector_prev.set_source_selector("src::skill-a", Some("selector-a".into()));
        let mut selector_next = selector_prev.clone();
        selector_next.set_source_selector("src::skill-a", Some("selector-b".into()));
        assert_eq!(
            selector_prev.lock_check(&selector_next),
            vec![LockDrift {
                status: DriftStatus::Updated,
                id: "selector::src::skill-a".into(),
            }]
        );
    }

    #[test]
    fn lock_check_flags_hash_and_revision_change() {
        let mut prev = AgentLockFile::default();
        prev.skills
            .insert("k::a".into(), skill_entry("d", "h1", "r1"));
        let mut next_hash = AgentLockFile::default();
        next_hash
            .skills
            .insert("k::a".into(), skill_entry("d", "h2", "r1"));
        assert_eq!(prev.lock_check(&next_hash)[0].status, DriftStatus::Updated);

        let mut next_rev = AgentLockFile::default();
        next_rev
            .skills
            .insert("k::a".into(), skill_entry("d", "h1", "r2"));
        assert_eq!(prev.lock_check(&next_rev)[0].status, DriftStatus::Updated);

        let mut same = AgentLockFile::default();
        same.skills
            .insert("k::a".into(), skill_entry("d", "h1", "r1"));
        assert!(prev.lock_check(&same).is_empty());
    }

    #[test]
    fn locked_mode_is_zero_network_and_never_resolves() {
        let mut prev = AgentLockFile::default();
        prev.skills
            .insert("src::a".into(), skill_entry("d", "h", "r"));

        assert!(!LockMode::Locked.allows_fetch());
        assert!(!LockMode::Locked.should_resolve("src", &prev));

        // Plain fetches + resolves everything.
        assert!(LockMode::Plain.allows_fetch());
        assert!(LockMode::Plain.should_resolve("src", &prev));
        assert!(LockMode::Plain.should_resolve("other", &prev));
    }

    #[test]
    fn update_mode_selective_resolve() {
        let mut prev = AgentLockFile::default();
        prev.skills.insert(
            "src::skill-a".into(),
            AgentLockEntry {
                skill: "skill-a".into(),
                source: "src".into(),
                ..Default::default()
            },
        );
        // Update of a named package re-resolves only its source; others carry over.
        let mode = LockMode::Update(vec!["skill-a".into()]);
        assert!(mode.allows_fetch());
        assert!(mode.should_resolve("src", &prev));
        assert!(!mode.should_resolve("unrelated-source", &prev));
        // Empty update list re-resolves all.
        assert!(LockMode::Update(vec![]).should_resolve("anything", &prev));
    }

    #[test]
    fn asset_helpers_filter_and_remove_by_kind() {
        let mut lock = AgentLockFile::default();
        lock.save_tracked_asset("mcp::a", test_asset("mcp", "a", "d1"));
        lock.save_tracked_asset("other::b", test_asset("other", "b", "d2"));
        lock.set_source_selector("mcp::a", Some("selector".into()));

        let mcps = lock.list_tracked_asset_ids("mcp");
        assert_eq!(mcps, vec![("mcp::a", "d1")]);

        lock.remove_tracked_asset("mcp::a");
        assert!(lock.get_tracked_asset("mcp", "mcp::a").is_none());
        assert!(!lock.source_selectors.contains_key("mcp::a"));

        lock.clear_all();
        assert!(lock.assets.is_empty());
        assert!(lock.source_selectors.is_empty());
    }

    #[test]
    fn state_round_trip_captures_skills() {
        let mut lock = AgentLockFile::default();
        lock.skills
            .insert("src::a".into(), skill_entry("d", "h", "r1"));
        lock.skills
            .insert("src::b".into(), skill_entry("d2", "h2", "r2"));
        lock.set_source_selector("src::a", Some("selector-a".into()));

        let state = lock.state();
        let mut blank = AgentLockFile::default();
        blank.apply_state(&state);

        assert_eq!(blank.version, LOCK_VERSION);
        assert_eq!(blank.skills.len(), 2);
        assert_eq!(blank.skills["src::a"].hash, "h");
        assert!(blank.skills["src::b"].destination == "d2");
        assert_eq!(blank.source_selectors["src::a"], "selector-a");
    }

    #[test]
    fn list_installed_commands_deduplicates_and_sorts() {
        let mut lock = AgentLockFile::default();
        lock.save_tracked_asset("cmd::a", test_asset("command", "z-cmd", "bin/z"));
        lock.save_tracked_asset("cmd::b", test_asset("command", "a-cmd", "bin/a"));
        lock.save_tracked_asset("cmd::c", test_asset("command", "a-cmd", "bin/a"));
        lock.save_tracked_asset("mcp::x", test_asset("mcp", "x", "srv"));

        assert_eq!(
            lock.list_installed_commands(),
            vec!["a-cmd".to_string(), "z-cmd".to_string()]
        );
    }

    #[test]
    fn list_installed_mcps_deduplicates_and_sorts() {
        let mut lock = AgentLockFile::default();
        let mut mcp = test_asset("mcp", "pack", "");
        mcp.destination = encode_asset_list(["bravo".into(), "alpha".into()]);
        lock.save_tracked_asset("mcp::a", mcp);
        let mut mcp2 = test_asset("mcp", "pack2", "");
        mcp2.destination = encode_asset_list(["alpha".into(), "charlie".into()]);
        lock.save_tracked_asset("mcp::b", mcp2);
        lock.save_tracked_asset("cmd::c", test_asset("command", "c", "bin/c"));

        assert_eq!(
            lock.list_installed_mcps().unwrap(),
            vec![
                "alpha".to_string(),
                "bravo".to_string(),
                "charlie".to_string()
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn lock_leaf_symlink_is_refused_without_touching_target() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("agent-env-lock-symlink");
        let outside = dir.join("outside");
        fs::write(&outside, "foreign").unwrap();
        let path = dir.join(LOCK_FILENAME);
        symlink(&outside, &path).unwrap();
        assert!(load(&path)
            .unwrap_err()
            .to_string()
            .contains("real regular file"));
        let mut lock = AgentLockFile::default();
        assert!(save(&mut lock, &path).is_err());
        assert_eq!(fs::read_to_string(&outside).unwrap(), "foreign");
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn lock_intermediate_parent_symlink_is_refused() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("agent-env-lock-parent-symlink");
        let outside = dir.join("outside");
        fs::create_dir(&outside).unwrap();
        let link = dir.join("linked");
        symlink(&outside, &link).unwrap();
        let mut lock = AgentLockFile::default();
        assert!(save(&mut lock, &link.join(LOCK_FILENAME)).is_err());
        assert!(!outside.join(LOCK_FILENAME).exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn scoped_save_tightens_global_mode_and_preserves_stricter_project_mode() {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_dir("agent-lock-mode-clamp");
        let global = dir.join("global.lock");
        fs::write(&global, "stale").unwrap();
        fs::set_permissions(&global, fs::Permissions::from_mode(0o644)).unwrap();
        let mut lock = AgentLockFile::default();
        save_for_scope(&mut lock, &global, Scope::Global).unwrap();
        assert_eq!(
            fs::metadata(&global).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let project = dir.join("project.lock");
        fs::write(&project, "stale").unwrap();
        fs::set_permissions(&project, fs::Permissions::from_mode(0o600)).unwrap();
        save_for_scope(&mut lock, &project, Scope::Project).unwrap();
        assert_eq!(
            fs::metadata(&project).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = fs::remove_dir_all(dir);
    }
}
