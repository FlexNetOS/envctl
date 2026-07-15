//! Pure, **Engine-free** verb drivers — the reusable orchestration helpers ported from
//! kasetto v3.2.0 `src/commands/{sync/{skills,commands,mcps},lock,list,clean,add}.rs`
//! (TASK-0013, Risk-1 split).
//!
//! These are the non-printing, non-`Engine` halves of the agent-asset verbs: they fetch /
//! hash / merge / prune and fill a [`SyncResult`] (counters + per-asset [`Action`] log),
//! but they emit no `Event`s and never `println!`. The envctl engine (`crates/engine/src/agent`)
//! wraps each one, turns the returned `actions` into `Event::AgentAction`s, and owns the
//! preview-vs-apply / Locked-zero-network policy by passing the right [`DriverCtx`].
//!
//! Faithfulness: the per-source fetch decision, the `--locked` fail-closed guard, and the
//! **never-prune-on-failure** rule (`remove_stale` only when `summary.failed == 0`) are ported
//! line-for-line. The MCP merge stays additive (`merge_mcp_config`) and `clean` only removes
//! lock-tracked MCP servers — pre-existing global servers are never touched.
//!
//! The kasetto `LockFile`/`State`/`SkillEntry` split collapses here onto the single
//! [`AgentLockFile`] (its `skills` map of [`AgentLockEntry`] *is* the old `State.skills`),
//! and the spinner/`ui` layer is dropped.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::agent::{
    all_command_global_targets, all_command_project_targets, all_mcp_project_targets,
    all_mcp_settings_targets, CommandTarget, McpSettingsTarget,
};
use crate::command::{
    destination_path, parse as parse_command, render as render_command, validate_command_name,
};
use crate::config::{
    CommandEntry, CommandsField, Config, McpEntry, McpsField, Scope, SkillTarget, SkillsField,
    SourceSpec, AGENT_PRESETS,
};
use crate::config_edit::{Pin, Section, Selector, SourceItem};
use crate::dirs::{dirs_agent_env_config, dirs_agent_env_data, dirs_home};
use crate::fsops::{
    relativize_dest, resolve_command_targets, resolve_dest, resolve_destinations,
    resolve_mcp_settings_targets, scope_root, select_targets,
};
use crate::hash::hash_bytes;
use crate::lock::{
    decode_asset_list, encode_asset_list, ensure_supported_version, installed_output_key,
    lock_path, AgentLockEntry, AgentLockFile, AssetEntry, InstalledOutput, LockMode, LOCK_VERSION,
};
use crate::mcp::{
    current_mcp_fragment_hashes, expected_mcp_fragment_hashes, mcp_server_names_from_bytes,
    render_mcp_settings_bytes,
};
use crate::profile::{format_updated_ago, read_skill_profile, read_skill_profile_from_dir};
use crate::report::{Action, InstalledSkill, Summary};
use crate::runtime::{runtime_state_path, ManagedOutput};
use crate::runtime_contract::validate_runtime_contract;
use crate::source::{
    derive_browse_url, discover_commands, discover_mcps, materialize_source, resolve_command_entry,
    resolve_mcp_entry, BrowseDerived,
};
use crate::sync::{
    command_action_label, command_asset_id, desired_command_names, desired_mcp_file_name_for_entry,
    desired_mcp_file_names, desired_skill_names, mcp_action_label, mcp_asset_id, skill_key,
};
use crate::util::{now_unix, now_unix_str};
use crate::{err, Result, TreeSnapshot};

static TRANSACTION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// The mutable bookkeeping threaded through a sync run (the lock, summary, and action log).
/// The kasetto `RuntimeState` updated-at timestamps live alongside the lock here.
#[derive(serde::Serialize)]
pub struct SyncResult {
    pub summary: Summary,
    pub actions: Vec<Action>,
}

impl SyncResult {
    fn new() -> Self {
        SyncResult {
            summary: Summary::default(),
            actions: Vec::new(),
        }
    }
}

/// Immutable per-run context for the sync driver (the engine builds this from the resolved
/// config + scope + the apply/lock-mode policy). Mirrors kasetto's `SyncContext` minus the
/// `ui`/`animate`/`plain`/`json` presentation flags (the engine emits Events instead).
pub struct DriverCtx<'a> {
    pub cfg: &'a Config,
    pub cfg_dir: &'a Path,
    pub destinations: &'a [PathBuf],
    pub scope_root: PathBuf,
    pub scope: Scope,
    /// `false` performs real writes; `true` is preview-only (no fetch-driven writes).
    pub dry_run: bool,
    /// `--update`: re-resolve moving refs and rewrite locked hashes.
    pub update: bool,
    /// `--update <name>...`: when non-empty, only sources providing these names are re-resolved.
    pub update_only: Vec<String>,
    /// `--locked`/`--frozen`: never fetch; error if the lock cannot satisfy the config.
    pub locked: bool,
}

impl DriverCtx<'_> {
    /// Build a `DriverCtx` from the resolved config + the lock mode + apply flag.
    pub fn from_mode<'a>(
        cfg: &'a Config,
        cfg_dir: &'a Path,
        destinations: &'a [PathBuf],
        scope_root: PathBuf,
        scope: Scope,
        apply: bool,
        lock_mode: &LockMode,
    ) -> DriverCtx<'a> {
        let (update, update_only, locked) = match lock_mode {
            LockMode::Plain => (false, Vec::new(), false),
            LockMode::Update(names) => (true, names.clone(), false),
            LockMode::Locked => (false, Vec::new(), true),
        };
        DriverCtx {
            cfg,
            cfg_dir,
            destinations,
            scope_root,
            scope,
            dry_run: !apply,
            update,
            update_only,
            locked,
        }
    }
}

/// A loaded runtime/updated-at memo for set/forget during a sync. Replaces the threading
/// of the `RuntimeState` into the kasetto per-kind helpers.
#[derive(Default)]
pub struct UpdatedAt {
    pub installed_at: BTreeMap<String, String>,
    pub managed_outputs: BTreeMap<String, ManagedOutput>,
    pub last_run: Option<String>,
    pub latest_report: Option<String>,
}

/// Drive a full sync (skills → commands → MCPs) against the lock, in place.
///
/// Returns the per-run [`SyncResult`]. The lock and `updated_at` memo are mutated when
/// `ctx.dry_run` is false; in preview mode nothing on disk or in the lock changes.
pub fn sync(ctx: &DriverCtx, lock: &mut AgentLockFile, updated: &mut UpdatedAt) -> SyncResult {
    if let Err(error) = ensure_supported_version(lock.version) {
        // A newer lock may carry fields this binary cannot deserialize. Do not persist even a
        // failure report: every lock/output/runtime byte must remain available to the newer owner.
        return sync_preflight_error(ctx.locked, error.to_string());
    }
    if lock.version != LOCK_VERSION && ctx.locked {
        return persist_coherent_failure_report(
            ctx,
            updated,
            sync_preflight_error(
                ctx.locked,
                format!(
                    "agent lock version {} cannot be used with tree-hash version {LOCK_VERSION}; fully rebuild the lock first",
                    lock.version
                ),
            ),
        );
    }
    if let Err(error) =
        validate_declared_ownership(ctx.cfg, ctx.cfg_dir, ctx.scope, lock, ctx.destinations)
    {
        return persist_coherent_failure_report(
            ctx,
            updated,
            sync_preflight_error(ctx.locked, error.to_string()),
        );
    }
    if ctx.locked {
        return match prepare_locked_sync(ctx, lock, updated) {
            Ok(plan) => apply_sync_plan(ctx, lock, updated, plan),
            Err(error) => persist_coherent_failure_report(
                ctx,
                updated,
                sync_preflight_error(true, error.to_string()),
            ),
        };
    }

    match prepare_nonlocked_sync(ctx, lock, updated) {
        Ok(plan) => apply_sync_plan(ctx, lock, updated, plan),
        Err(error) => persist_coherent_failure_report(
            ctx,
            updated,
            sync_preflight_error(false, error.to_string()),
        ),
    }
}

fn sync_preflight_error(locked: bool, message: String) -> SyncResult {
    let mut result = SyncResult::new();
    result.summary.failed = 1;
    result.actions.push(Action {
        source: None,
        skill: None,
        status: if locked {
            "locked_error".into()
        } else {
            "source_error".into()
        },
        error: Some(message),
    });
    result
}

fn persist_coherent_failure_report(
    ctx: &DriverCtx,
    updated: &mut UpdatedAt,
    mut failure: SyncResult,
) -> SyncResult {
    if ctx.dry_run {
        return failure;
    }
    if let Ok(report) = serde_json::to_string(&failure) {
        let runtime = crate::runtime::RuntimeState {
            last_run: updated.last_run.clone(),
            latest_report: Some(report.clone()),
            installed_at: updated.installed_at.clone(),
            managed_outputs: updated.managed_outputs.clone(),
        };
        match crate::runtime::save_runtime_state(&runtime, ctx.scope, ctx.cfg_dir) {
            Ok(()) => updated.latest_report = Some(report),
            Err(persist) => {
                if let Some(action) = failure.actions.first_mut() {
                    let message = action.error.get_or_insert_default();
                    message.push_str(&format!(
                        "; failed to persist coherent failure report: {persist}"
                    ));
                }
            }
        }
    }
    failure
}

fn validate_safe_segment(kind: &str, name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
        || Path::new(name).is_absolute()
    {
        return Err(err(format!(
            "invalid {kind} name `{name}`: expected one safe path segment"
        )));
    }
    Ok(())
}

fn validate_id_source(kind: &str, source: &str) -> Result<()> {
    if source.is_empty() || source.contains('\0') {
        return Err(err(format!(
            "invalid {kind} source: source identities must be nonempty and contain no NUL"
        )));
    }
    Ok(())
}

fn claim_id(ids: &mut HashSet<String>, id: String) -> Result<()> {
    if !ids.insert(id.clone()) {
        return Err(err(format!("duplicate computed agent asset id `{id}`")));
    }
    Ok(())
}

fn claim_destination(
    destinations: &mut HashMap<PathBuf, String>,
    path: PathBuf,
    owner: &str,
) -> Result<()> {
    if let Some(existing) = destinations.get(&path) {
        return Err(err(format!(
            "duplicate agent asset destination claim at {} between `{existing}` and `{owner}`",
            path.display()
        )));
    }
    destinations.insert(path, owner.to_string());
    Ok(())
}

/// Validate identities and all statically-resolvable destination ownership before any write.
fn validate_declared_ownership(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
    skill_destinations: &[PathBuf],
) -> Result<()> {
    validate_runtime_contract(cfg.runtime)?;
    validate_portable_output_claims(
        prev,
        scope,
        &scope_root(scope, cfg_dir)?,
        skill_destinations,
    )?;
    validate_raw_agent_targets(cfg, cfg_dir, scope)?;
    let command_targets = resolve_command_targets(cfg, scope, cfg_dir)?;
    let mcp_targets = resolve_mcp_settings_targets(cfg, scope, cfg_dir)?;
    let mut ids = HashSet::new();
    let mut destinations = HashMap::new();

    for destination in skill_destinations {
        validate_managed_destination(destination, scope, &scope_root(scope, cfg_dir)?)?;
    }
    for target in &command_targets {
        validate_managed_destination(&target.path, scope, &scope_root(scope, cfg_dir)?)?;
    }
    for target in &mcp_targets {
        validate_managed_destination(&target.path, scope, &scope_root(scope, cfg_dir)?)?;
    }

    for source in &cfg.skills {
        validate_id_source("skill", &source.source)?;
        for name in desired_skill_names(source, prev) {
            validate_safe_segment("skill", &name)?;
            let id = skill_key(&source.source, &name);
            claim_id(&mut ids, id.clone())?;
            for destination in skill_destinations {
                claim_destination(&mut destinations, destination.join(&name), &id)?;
            }
        }
    }

    for source in &cfg.commands {
        validate_id_source("command", &source.source)?;
        for name in desired_command_names(source, prev) {
            validate_command_name(&name)?;
            let id = command_asset_id(&source.source, &name);
            claim_id(&mut ids, id.clone())?;
            for target in &command_targets {
                claim_destination(&mut destinations, destination_path(target, &name), &id)?;
            }
        }
    }

    for source in &cfg.mcps {
        validate_id_source("MCP", &source.source)?;
        for file_name in desired_mcp_file_names(source, prev) {
            validate_safe_segment("MCP pack file", &file_name)?;
            claim_id(&mut ids, mcp_asset_id(&source.source, &file_name))?;
        }
    }
    Ok(())
}

fn validate_managed_destination(path: &Path, scope: Scope, root: &Path) -> Result<()> {
    if scope == Scope::Project {
        let relative = path.strip_prefix(root).map_err(|_| {
            err(format!(
                "project managed destination escapes the project root: {}",
                path.display()
            ))
        })?;
        if relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return Err(err(format!(
                "project managed destination contains an unsafe component: {}",
                path.display()
            )));
        }
    }

    let parent = if fs::symlink_metadata(path).is_ok() {
        path
    } else {
        path.parent()
            .ok_or_else(|| err("managed destination has no parent"))?
    };
    let mut cursor = parent;
    loop {
        match fs::symlink_metadata(cursor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(err(format!(
                        "managed destination traverses a symlink: {}",
                        cursor.display()
                    )));
                }
                if cursor == root || cursor.starts_with(root) {
                    ensure_current_user_owner(&metadata, cursor)?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        if cursor == root {
            break;
        }
        cursor = cursor.parent().ok_or_else(|| {
            err(format!(
                "managed destination is not rooted at {}",
                root.display()
            ))
        })?;
        if !cursor.starts_with(root) && cursor != root {
            if scope == Scope::Project {
                return Err(err(format!(
                    "managed destination escapes the project root: {}",
                    path.display()
                )));
            }
            // Global custom destinations may sit outside HOME; inspect their existing chain to
            // filesystem root for symlinks, but do not impose ownership on system ancestors.
            if cursor.parent().is_none() {
                break;
            }
        }
    }
    Ok(())
}

fn validate_raw_agent_targets(cfg: &Config, cfg_dir: &Path, scope: Scope) -> Result<()> {
    let agents = cfg.agents();
    let mut unique_agents = HashSet::new();
    for agent in &agents {
        if !unique_agents.insert(*agent) {
            return Err(err(format!(
                "duplicate configured agent target: `{agent:?}`"
            )));
        }
    }

    let home = if scope == Scope::Global {
        Some(dirs_home()?)
    } else {
        None
    };
    let agent_env_config = if scope == Scope::Global {
        Some(dirs_agent_env_config()?)
    } else {
        None
    };
    let mut claims = HashMap::<(String, PathBuf), String>::new();
    let mut claim = |surface: &str, path: PathBuf, owner: String| -> Result<()> {
        let key = (surface.to_string(), path.clone());
        if let Some(previous) = claims.insert(key, owner.clone()) {
            return Err(err(format!(
                "duplicate raw {surface} target claim at {} between `{previous}` and `{owner}`",
                path.display()
            )));
        }
        Ok(())
    };

    for agent in agents {
        if cfg.destination.is_none() {
            let paths = match scope {
                Scope::Project => agent.project_skill_paths(cfg_dir),
                Scope::Global => vec![agent.global_path(home.as_deref().expect("global home"))],
            };
            for path in paths {
                claim("skill", path, format!("{agent:?}"))?;
            }
        }
        let mcp = match scope {
            Scope::Project => agent.mcp_project_target(cfg_dir),
            Scope::Global => agent.mcp_settings_target(
                home.as_deref().expect("global home"),
                agent_env_config.as_deref().expect("global config"),
            ),
        };
        claim(
            "MCP",
            mcp.path,
            format!("{agent:?}@{}", mcp_format_binding(mcp.format)),
        )?;
        let command = match scope {
            Scope::Project => agent.commands_project_path(cfg_dir),
            Scope::Global => agent.commands_global_path(home.as_deref().expect("global home")),
        };
        if let Some(command) = command {
            claim(
                "command",
                command.path,
                format!("{agent:?}@{}", command_format_binding(command.format)),
            )?;
        }
    }
    Ok(())
}

fn validate_portable_output_claims(
    lock: &AgentLockFile,
    scope: Scope,
    root: &Path,
    configured_skill_roots: &[PathBuf],
) -> Result<()> {
    if scope != Scope::Project && !lock.installed_outputs.is_empty() {
        return Err(err(
            "portable installed-output claims are valid only in a project-scope lock",
        ));
    }
    let mut claims = HashMap::<(PathBuf, String, String), String>::new();
    for (key, proof) in &lock.installed_outputs {
        if installed_output_key(
            &proof.asset_id,
            &proof.destination,
            &proof.format,
            &proof.unit,
        ) != *key
        {
            return Err(err(format!(
                "installed-output key does not match its framed identity: `{key}`"
            )));
        }
        let destination = Path::new(&proof.destination);
        let mut normalized = PathBuf::new();
        let canonical = !proof.destination.is_empty()
            && !destination.is_absolute()
            && destination.components().all(|component| match component {
                std::path::Component::Normal(value) => {
                    normalized.push(value);
                    true
                }
                _ => false,
            })
            && normalized.to_string_lossy() == proof.destination;
        if !canonical
            || proof.asset_id.contains('\0')
            || proof.format.contains('\0')
            || proof.unit.contains('\0')
        {
            return Err(err(format!(
                "installed-output claim has an unsafe identity: `{key}`"
            )));
        }
        if scope == Scope::Project
            && !portable_claim_target_allowed(proof, root, configured_skill_roots)?
            && !portable_historical_skill_claim_shape(proof, root)?
        {
            return Err(err(format!(
                "installed-output claim is outside known native managed targets: `{key}`"
            )));
        }
        let claim = (normalized, proof.format.clone(), proof.unit.clone());
        if let Some(previous) = claims.insert(claim, proof.asset_id.clone()) {
            if previous != proof.asset_id {
                return Err(err(format!(
                    "installed-output unit `{}` at `{}` is claimed by both `{previous}` and `{}`",
                    proof.unit, proof.destination, proof.asset_id
                )));
            }
        }
    }
    Ok(())
}

fn parse_framed_asset_id<'a>(kind: &str, id: &'a str) -> Option<(&'a str, &'a str)> {
    let mut rest = id.strip_prefix(&format!("{kind}::v3|"))?;
    let colon = rest.find(':')?;
    let source_len = rest[..colon].parse::<usize>().ok()?;
    rest = &rest[colon + 1..];
    if rest.len() < source_len || !rest.is_char_boundary(source_len) {
        return None;
    }
    let (source, tail) = rest.split_at(source_len);
    rest = tail.strip_prefix('|')?;
    let colon = rest.find(':')?;
    let name_len = rest[..colon].parse::<usize>().ok()?;
    rest = &rest[colon + 1..];
    if rest.len() != name_len || !rest.is_char_boundary(name_len) {
        return None;
    }
    Some((source, rest))
}

fn portable_claim_target_allowed(
    proof: &InstalledOutput,
    root: &Path,
    configured_skill_roots: &[PathBuf],
) -> Result<bool> {
    let destination = resolve_dest(&proof.destination, root);
    if proof.format == "skill-tree" && proof.unit == "tree" {
        let Some((_, name)) = parse_framed_asset_id("skill", &proof.asset_id) else {
            return Ok(false);
        };
        validate_safe_segment("skill", name)?;
        return Ok(AGENT_PRESETS
            .iter()
            .flat_map(|agent| agent.project_skill_paths(root))
            .any(|skill_root| skill_root.join(name) == destination)
            || configured_skill_roots
                .iter()
                .any(|skill_root| skill_root.join(name) == destination));
    }
    if !is_mcp_output_format(&proof.format) {
        let Some((_, name)) = parse_framed_asset_id("command", &proof.asset_id) else {
            return Ok(false);
        };
        validate_command_name(name)?;
        return Ok(all_command_project_targets(root).iter().any(|target| {
            command_format_binding(target.format) == proof.format
                && destination_path(target, name) == destination
                && proof.unit == "file"
        }));
    }
    if parse_framed_asset_id("mcp", &proof.asset_id).is_none() {
        return Ok(false);
    }
    Ok(all_mcp_project_targets(root).iter().any(|target| {
        target.path == destination && mcp_format_binding(target.format) == proof.format
    }))
}

fn portable_historical_skill_claim_shape(proof: &InstalledOutput, root: &Path) -> Result<bool> {
    if proof.format != "skill-tree" || proof.unit != "tree" {
        return Ok(false);
    }
    let Some((_, name)) = parse_framed_asset_id("skill", &proof.asset_id) else {
        return Ok(false);
    };
    validate_safe_segment("skill", name)?;
    let destination = resolve_dest(&proof.destination, root);
    Ok(
        destination.file_name().and_then(|value| value.to_str()) == Some(name)
            && destination.starts_with(root)
            && destination != root,
    )
}

fn validate_historical_custom_skill_claims(
    lock: &AgentLockFile,
    updated: &UpdatedAt,
    root: &Path,
    configured_skill_roots: &[PathBuf],
) -> Result<()> {
    for proof in lock.installed_outputs.values() {
        if portable_claim_target_allowed(proof, root, configured_skill_roots)? {
            continue;
        }
        if !portable_historical_skill_claim_shape(proof, root)? {
            continue;
        }
        let destination = resolve_dest(&proof.destination, root);
        let destination_text = destination.to_string_lossy();
        let key = managed_output_key(
            &proof.asset_id,
            &destination_text,
            &proof.format,
            &proof.unit,
        );
        let Some(runtime) = updated.managed_outputs.get(&key) else {
            return Err(err(format!(
                "historical custom skill target requires its machine ownership proof: {}",
                destination.display()
            )));
        };
        if runtime.asset_id != proof.asset_id
            || runtime.destination != destination_text
            || runtime.format != proof.format
            || runtime.unit != proof.unit
            || runtime.hash != proof.hash
            || managed_output_key(
                &runtime.asset_id,
                &runtime.destination,
                &runtime.format,
                &runtime.unit,
            ) != key
        {
            return Err(err(format!(
                "historical custom skill ownership proof drift at {}",
                destination.display()
            )));
        }
    }
    Ok(())
}

/// Validate the complete locked input set before any destination or lock mutation.
///
/// The zero-network auditor re-hashes local source bytes and reconstructs every portable
/// identity field (selector, destination, revision, and scope). Remote entries are carried
/// only after their existing exact pins/selectors are proven; they are never materialized.
/// Comparing that complete expected snapshot before the first copy keeps a later local-source
/// mismatch from leaving earlier asset kinds partially installed.
fn validate_locked_snapshot(lock: &AgentLockFile, expected: &AgentLockFile) -> Result<()> {
    let drift = lock.lock_check(expected);
    if drift.is_empty() {
        return Ok(());
    }

    let details = drift
        .iter()
        .map(|item| format!("{} {}", item.status.label(), item.id))
        .collect::<Vec<_>>()
        .join(", ");
    Err(err(format!("--locked: lock drift ({details})")))
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum OriginalState {
    Missing,
    File(String, u32),
    Directory(String, bool),
}

#[derive(Clone)]
enum ReplacementPayload {
    File { bytes: Vec<u8>, new_mode: u32 },
    Directory(TreeSnapshot),
    Delete,
}

#[derive(Clone)]
struct LockedReplacement {
    destination: PathBuf,
    original: OriginalState,
    payload: ReplacementPayload,
}

struct LockedSyncPlan {
    result: SyncResult,
    replacements: Vec<LockedReplacement>,
    managed_outputs: BTreeMap<String, ManagedOutput>,
    resulting_lock: Option<AgentLockFile>,
    final_installed_at: BTreeMap<String, String>,
    final_last_run: Option<String>,
    final_latest_report: Option<String>,
}

fn managed_output_key(asset_id: &str, destination: &str, format: &str, unit: &str) -> String {
    format!(
        "{}:{asset_id}|{}:{destination}|{}:{format}|{}:{unit}",
        asset_id.len(),
        destination.len(),
        format.len(),
        unit.len()
    )
}

fn relative_project_destination(destination: &Path, root: &Path) -> Result<Option<String>> {
    let Ok(relative) = destination.strip_prefix(root) else {
        return Ok(None);
    };
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(value) => normalized.push(value),
            std::path::Component::CurDir => {}
            _ => {
                return Err(err(format!(
                    "managed output is not a safe project-relative path: {}",
                    destination.display()
                )));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(err(format!(
            "managed output is not a safe project-relative path: {}",
            destination.display()
        )));
    }
    Ok(Some(normalized.to_string_lossy().into_owned()))
}

fn portable_output_claim(
    asset_id: &str,
    destination: &Path,
    scope: Scope,
    root: &Path,
    format: &str,
    unit: &str,
    hash: &str,
) -> Result<Option<(String, InstalledOutput)>> {
    if scope != Scope::Project {
        return Ok(None);
    }
    let Some(destination) = relative_project_destination(destination, root)? else {
        return Ok(None);
    };
    let key = installed_output_key(asset_id, &destination, format, unit);
    Ok(Some((
        key,
        InstalledOutput {
            asset_id: asset_id.to_string(),
            destination,
            format: format.to_string(),
            unit: unit.to_string(),
            hash: hash.to_string(),
        },
    )))
}

fn managed_output(
    asset_id: &str,
    destination: &Path,
    format: &str,
    unit: &str,
    hash: &str,
) -> (String, ManagedOutput) {
    let destination = destination.to_string_lossy().into_owned();
    let key = managed_output_key(asset_id, &destination, format, unit);
    (
        key,
        ManagedOutput {
            asset_id: asset_id.to_string(),
            destination,
            format: format.to_string(),
            unit: unit.to_string(),
            hash: hash.to_string(),
        },
    )
}

fn inspect_file_state(path: &Path) -> Result<OriginalState> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(OriginalState::Missing),
        Err(error) => Err(error.into()),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(err(format!(
                "managed file destination must be a real file: {}",
                path.display()
            )))
        }
        Ok(metadata) => {
            ensure_current_user_owner(&metadata, path)?;
            Ok(OriginalState::File(
                hash_bytes(&fs::read(path)?),
                file_permission_mode(&metadata),
            ))
        }
    }
}

#[cfg(unix)]
fn file_permission_mode(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o7777
}

#[cfg(not(unix))]
fn file_permission_mode(_metadata: &fs::Metadata) -> u32 {
    0
}

fn inspect_directory_state(path: &Path) -> Result<OriginalState> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(OriginalState::Missing),
        Err(error) => Err(error.into()),
        Ok(_) => Ok(OriginalState::Directory(
            TreeSnapshot::capture_destination(path)?.hash(),
            false,
        )),
    }
}

fn skill_snapshot_for_scope(snapshot: TreeSnapshot, scope: Scope) -> Result<TreeSnapshot> {
    if scope == Scope::Project {
        snapshot.into_git_portable()
    } else {
        Ok(snapshot)
    }
}

fn inspect_skill_directory_state(path: &Path, scope: Scope) -> Result<OriginalState> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(OriginalState::Missing),
        Err(error) => Err(error.into()),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(err(format!(
                "managed skill destination must be a real directory: {}",
                path.display()
            )))
        }
        Ok(metadata) => {
            ensure_current_user_owner(&metadata, path)?;
            let snapshot =
                skill_snapshot_for_scope(TreeSnapshot::capture_destination(path)?, scope)?;
            Ok(OriginalState::Directory(
                snapshot.hash(),
                scope == Scope::Project,
            ))
        }
    }
}

#[cfg(unix)]
fn ensure_current_user_owner(metadata: &fs::Metadata, path: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let status = fs::read_to_string("/proc/self/status")?;
    let uid = status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|fields| fields.split_whitespace().next())
        .and_then(|field| field.parse::<u32>().ok())
        .ok_or_else(|| err("cannot determine current uid"))?;
    if metadata.uid() != uid {
        return Err(err(format!(
            "managed destination is not current-user-owned: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_current_user_owner(_metadata: &fs::Metadata, _path: &Path) -> Result<()> {
    Ok(())
}

// The arguments mirror every field of the ownership tuple plus its two proof stores. Keeping
// them explicit makes accidental identity omission visible at each destructive call site.
#[allow(clippy::too_many_arguments)]
fn require_output_ownership(
    lock: &AgentLockFile,
    updated: &UpdatedAt,
    asset_id: &str,
    destination: &Path,
    scope: Scope,
    root: &Path,
    format: &str,
    unit: &str,
    current_hash: &str,
) -> Result<()> {
    if scope == Scope::Project {
        if let Some((portable_key, expected)) = portable_output_claim(
            asset_id,
            destination,
            scope,
            root,
            format,
            unit,
            current_hash,
        )? {
            if let Some(proof) = lock.installed_outputs.get(&portable_key) {
                if installed_output_key(
                    &proof.asset_id,
                    &proof.destination,
                    &proof.format,
                    &proof.unit,
                ) != portable_key
                    || proof != &expected
                {
                    return Err(err(format!(
                        "portable installed-output proof drift at {}",
                        destination.display()
                    )));
                }
                // The checked-in project proof is authoritative. A disposable machine cache
                // may be stale after a clone or interrupted cache refresh and must not veto it.
                return Ok(());
            }
        }
    }
    let destination_text = destination.to_string_lossy();
    let key = managed_output_key(asset_id, &destination_text, format, unit);
    if let Some(proof) = updated.managed_outputs.get(&key) {
        if managed_output_key(
            &proof.asset_id,
            &proof.destination,
            &proof.format,
            &proof.unit,
        ) != key
            || proof.asset_id != asset_id
            || proof.destination != destination_text
            || proof.format != format
            || proof.unit != unit
        {
            return Err(err(format!(
                "invalid managed-output proof identity at {}",
                destination.display()
            )));
        }
        if proof.hash != current_hash {
            return Err(err(format!(
                "managed-output proof drift at {}",
                destination.display()
            )));
        }
        return Ok(());
    }
    Err(err(format!(
        "refusing unowned pre-existing output at {}",
        destination.display()
    )))
}

/// A project lock is the portable ownership root.  If a previous apply was interrupted after
/// copying a correct output but before it rewrote that root, the proof's hash can be stale while
/// the live, normalized tree is exactly the locked desired tree.  A normal (lock-writing) sync
/// may repair that attestation; a locked audit deliberately remains fail-closed.
///
/// This is intentionally narrower than ordinary ownership acceptance: every identity field must
/// still match, the output must already equal the verified desired hash, and the caller must be
/// preparing a transaction that persists the replacement proof.
#[allow(clippy::too_many_arguments)]
fn portable_output_proof_is_reattestable(
    lock: &AgentLockFile,
    asset_id: &str,
    destination: &Path,
    scope: Scope,
    root: &Path,
    format: &str,
    unit: &str,
    current_hash: &str,
    desired_hash: &str,
    persist_lock: bool,
) -> Result<bool> {
    if !persist_lock || current_hash != desired_hash {
        return Ok(false);
    }
    let Some((portable_key, expected)) = portable_output_claim(
        asset_id,
        destination,
        scope,
        root,
        format,
        unit,
        current_hash,
    )?
    else {
        return Ok(false);
    };
    let Some(proof) = lock.installed_outputs.get(&portable_key) else {
        return Ok(false);
    };
    Ok(proof.hash != current_hash
        && installed_output_key(
            &proof.asset_id,
            &proof.destination,
            &proof.format,
            &proof.unit,
        ) == portable_key
        && proof.asset_id == expected.asset_id
        && proof.destination == expected.destination
        && proof.format == expected.format
        && proof.unit == expected.unit)
}

#[cfg(test)]
fn record_runtime_ownership(
    updated: &mut UpdatedAt,
    asset_id: &str,
    destination: &Path,
    format: &str,
    unit: &str,
    hash: &str,
) {
    let (key, proof) = managed_output(asset_id, destination, format, unit, hash);
    updated.managed_outputs.insert(key, proof);
}

fn asset_was_managed(lock: &AgentLockFile, updated: &UpdatedAt, asset_id: &str) -> bool {
    lock.installed_outputs
        .values()
        .any(|proof| proof.asset_id == asset_id)
        || updated
            .managed_outputs
            .values()
            .any(|proof| proof.asset_id == asset_id)
}

fn legacy_v2_exact_output_is_named(
    lock: &AgentLockFile,
    asset_id: &str,
    destination: &Path,
    ctx: &DriverCtx,
    unit: &str,
    current_hash: &str,
    desired_hash: &str,
) -> Result<bool> {
    if lock.version >= LOCK_VERSION || current_hash != desired_hash {
        return Ok(false);
    }
    let legacy_skill = lock.skills.get(asset_id).or_else(|| {
        let (source, name) = parse_framed_asset_id("skill", asset_id)?;
        lock.skills
            .values()
            .find(|entry| entry.source == source && entry.skill == name)
    });
    if let Some(entry) = legacy_skill {
        return Ok(
            resolve_dest(&entry.destination, &ctx.scope_root) == destination
                || ctx
                    .destinations
                    .iter()
                    .any(|root| root.join(&entry.skill) == destination),
        );
    }
    let legacy_asset = lock.assets.get(asset_id).or_else(|| {
        ["command", "mcp"].into_iter().find_map(|kind| {
            let (source, name) = parse_framed_asset_id(kind, asset_id)?;
            lock.assets
                .values()
                .find(|entry| entry.kind == kind && entry.source == source && entry.name == name)
        })
    });
    let Some(entry) = legacy_asset else {
        return Ok(false);
    };
    let named = decode_asset_list(&entry.destination, lock.version)?;
    match entry.kind.as_str() {
        "command" => Ok(named
            .iter()
            .map(|path| resolve_dest(path, &ctx.scope_root))
            .any(|path| path == destination)),
        // v2 MCP locks named the owned server units while config binds the native targets.
        "mcp" => Ok(named.iter().any(|name| name == unit)),
        _ => Ok(false),
    }
}

fn replace_managed_target(
    managed: &mut BTreeMap<String, ManagedOutput>,
    asset_id: &str,
    destination: &Path,
    format: &str,
) {
    let destination = destination.to_string_lossy();
    managed.retain(|_, output| {
        output.asset_id != asset_id || output.destination != destination || output.format != format
    });
}

fn historical_mcp_proofs(
    lock: &AgentLockFile,
    updated: &UpdatedAt,
    ctx: &DriverCtx,
    asset_id: &str,
    target: &McpSettingsTarget,
    format: &str,
) -> Result<Vec<ManagedOutput>> {
    let mut proofs = BTreeMap::<String, ManagedOutput>::new();
    if ctx.scope == Scope::Project {
        for proof in lock.installed_outputs.values().filter(|proof| {
            proof.asset_id == asset_id
                && proof.format == format
                && resolve_dest(&proof.destination, &ctx.scope_root) == target.path
        }) {
            proofs.insert(
                proof.unit.clone(),
                ManagedOutput {
                    asset_id: proof.asset_id.clone(),
                    destination: target.path.to_string_lossy().into_owned(),
                    format: proof.format.clone(),
                    unit: proof.unit.clone(),
                    hash: proof.hash.clone(),
                },
            );
        }
    }
    for (key, proof) in &updated.managed_outputs {
        if proof.asset_id != asset_id
            || proof.destination != target.path.to_string_lossy()
            || proof.format != format
        {
            continue;
        }
        if managed_output_key(
            &proof.asset_id,
            &proof.destination,
            &proof.format,
            &proof.unit,
        ) != *key
        {
            return Err(err(format!(
                "invalid managed MCP proof identity at {}",
                target.path.display()
            )));
        }
        proofs
            .entry(proof.unit.clone())
            .or_insert_with(|| proof.clone());
    }
    Ok(proofs.into_values().collect())
}

fn validate_runtime_proof_key(key: &str, proof: &ManagedOutput) -> Result<()> {
    if managed_output_key(
        &proof.asset_id,
        &proof.destination,
        &proof.format,
        &proof.unit,
    ) != key
    {
        return Err(err(format!(
            "managed-output key does not match its framed identity: `{key}`"
        )));
    }
    Ok(())
}

fn global_runtime_output_allowed(proof: &ManagedOutput) -> Result<bool> {
    let destination = Path::new(&proof.destination);
    let home = dirs_home()?;
    if proof.format == "skill-tree" && proof.unit == "tree" {
        let Some((_, name)) = parse_framed_asset_id("skill", &proof.asset_id) else {
            return Ok(false);
        };
        validate_safe_segment("skill", name)?;
        let native = AGENT_PRESETS
            .iter()
            .any(|agent| agent.global_path(&home).join(name) == destination);
        // A machine-local runtime proof is the authority for a retired global custom root. The
        // external path remains bounded to the exact framed skill identity and leaf name; content
        // and current-user ownership are checked before mutation.
        return Ok(native
            || (destination.is_absolute()
                && destination.file_name().and_then(|value| value.to_str()) == Some(name)
                && destination.parent().is_some()));
    }
    let Some((_, name)) = parse_framed_asset_id("command", &proof.asset_id) else {
        return Ok(false);
    };
    validate_command_name(name)?;
    Ok(all_command_global_targets(&home).iter().any(|target| {
        destination_path(target, name) == destination
            && proof.format == command_format_binding(target.format)
            && proof.unit == "file"
    }))
}

fn prepare_locked_sync(
    ctx: &DriverCtx,
    lock: &AgentLockFile,
    updated: &UpdatedAt,
) -> Result<LockedSyncPlan> {
    let (expected, inputs) = build_zero_network_snapshot(ctx.cfg, ctx.cfg_dir, ctx.scope, lock)?;
    validate_locked_snapshot(lock, &expected)?;
    prepare_sync_plan(ctx, lock, expected, inputs, updated, false, false, false)
}

fn prepare_nonlocked_sync(
    ctx: &DriverCtx,
    lock: &AgentLockFile,
    updated: &UpdatedAt,
) -> Result<LockedSyncPlan> {
    let legacy_migration = lock.version < LOCK_VERSION;
    let desired = rebuild_lock(
        ctx.cfg,
        ctx.cfg_dir,
        ctx.scope,
        lock,
        if ctx.update { &ctx.update_only } else { &[] },
    )?;
    let (audited, mut inputs) =
        build_zero_network_snapshot(ctx.cfg, ctx.cfg_dir, ctx.scope, &desired)?;
    validate_locked_snapshot(&desired, &audited)?;
    materialize_remote_inputs(ctx, &desired, &mut inputs)?;
    prepare_sync_plan(
        ctx,
        lock,
        desired,
        inputs,
        updated,
        true,
        true,
        legacy_migration,
    )
}

fn materialize_remote_inputs(
    ctx: &DriverCtx,
    desired: &AgentLockFile,
    inputs: &mut LockedInputs,
) -> Result<()> {
    materialize_remote_inputs_with(ctx, desired, inputs, &materialize_source)
}

fn materialize_remote_inputs_with(
    ctx: &DriverCtx,
    desired: &AgentLockFile,
    inputs: &mut LockedInputs,
    materialize: &dyn Fn(&SourceSpec, &Path, &Path) -> Result<crate::source::MaterializedSource>,
) -> Result<()> {
    for (index, source) in ctx.cfg.skills.iter().enumerate() {
        if !source.source.contains("://") {
            continue;
        }
        let stage = std::env::temp_dir().join(format!(
            "envctl-agent-sync-input-skill-{}-{index}",
            transaction_suffix()
        ));
        let materialized = materialize(source, ctx.cfg_dir, &stage)?;
        let result = (|| {
            let (targets, broken) = select_targets(
                &source.skills,
                &materialized.available,
                &materialized.source_root,
            )?;
            if let Some(broken) = broken.first() {
                return Err(err(format!(
                    "skill `{}` not found in {}",
                    broken.name, source.source
                )));
            }
            for (name, path) in targets {
                let id = skill_key(&source.source, &name);
                let snapshot = skill_snapshot_for_scope(
                    TreeSnapshot::capture_within(&materialized.source_root, &path)?,
                    ctx.scope,
                )?;
                let expected = desired.skills.get(&id).ok_or_else(|| {
                    err(format!(
                        "materialized unexpected skill `{name}` from {}",
                        source.source
                    ))
                })?;
                if expected.hash != snapshot.hash()
                    || expected.source_revision != materialized.source_revision
                {
                    return Err(err(format!(
                        "materialized skill `{name}` changed between lock preparation and staging"
                    )));
                }
                inputs.skills.insert(id, snapshot);
            }
            Ok(())
        })();
        if let Some(cleanup) = materialized.cleanup_dir {
            let _ = force_remove(&cleanup);
        }
        result?;
    }

    for (index, source) in ctx.cfg.commands.iter().enumerate() {
        if !source.source.contains("://") {
            continue;
        }
        let stage = std::env::temp_dir().join(format!(
            "envctl-agent-sync-input-command-{}-{index}",
            transaction_suffix()
        ));
        let source_spec = source.as_source_spec();
        let materialized = materialize(&source_spec, ctx.cfg_dir, &stage)?;
        let result = (|| {
            for (name, path) in select_commands_for_audit(source, &materialized.source_root)? {
                let id = command_asset_id(&source.source, &name);
                let bytes = TreeSnapshot::capture_file_within(&materialized.source_root, &path)?;
                let text = String::from_utf8(bytes.clone())
                    .map_err(|error| err(format!("command source is not UTF-8: {error}")))?;
                parse_command(&text)?;
                let expected = desired.assets.get(&id).ok_or_else(|| {
                    err(format!(
                        "materialized unexpected command `{name}` from {}",
                        source.source
                    ))
                })?;
                if expected.hash != hash_bytes(&bytes)
                    || expected.source_revision != materialized.source_revision
                {
                    return Err(err(format!(
                        "materialized command `{name}` changed between lock preparation and staging"
                    )));
                }
                inputs.commands.insert(id, text);
            }
            Ok(())
        })();
        if let Some(cleanup) = materialized.cleanup_dir {
            let _ = force_remove(&cleanup);
        }
        result?;
    }

    for (index, source) in ctx.cfg.mcps.iter().enumerate() {
        if !source.source.contains("://") {
            continue;
        }
        let stage = std::env::temp_dir().join(format!(
            "envctl-agent-sync-input-mcp-{}-{index}",
            transaction_suffix()
        ));
        let source_spec = source.as_source_spec();
        let materialized = materialize(&source_spec, ctx.cfg_dir, &stage)?;
        let result = (|| {
            for path in select_mcps_for_audit(source, &materialized.source_root)? {
                let file_name = file_name_str(&path);
                let id = mcp_asset_id(&source.source, &file_name);
                let bytes = TreeSnapshot::capture_file_within(&materialized.source_root, &path)?;
                let expected = desired.assets.get(&id).ok_or_else(|| {
                    err(format!(
                        "materialized unexpected MCP `{file_name}` from {}",
                        source.source
                    ))
                })?;
                if expected.hash != hash_bytes(&bytes)
                    || expected.source_revision != materialized.source_revision
                {
                    return Err(err(format!(
                        "materialized MCP `{file_name}` changed between lock preparation and staging"
                    )));
                }
                inputs.mcps.insert(id, bytes);
            }
            Ok(())
        })();
        if let Some(cleanup) = materialized.cleanup_dir {
            let _ = force_remove(&cleanup);
        }
        result?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn prepare_sync_plan(
    ctx: &DriverCtx,
    lock: &AgentLockFile,
    mut expected: AgentLockFile,
    inputs: LockedInputs,
    updated: &UpdatedAt,
    allow_unattested_missing: bool,
    persist_lock: bool,
    legacy_migration: bool,
) -> Result<LockedSyncPlan> {
    if ctx.scope == Scope::Project {
        validate_historical_custom_skill_claims(lock, updated, &ctx.scope_root, ctx.destinations)?;
    }
    let mut result = SyncResult::new();
    let mut replacements = Vec::new();
    let mut managed = updated.managed_outputs.clone();
    let mut active_output_keys = HashSet::new();
    let mut active_portable_keys = HashSet::new();
    let mut updated_ids = Vec::new();
    let mut removed_proof_assets = HashSet::new();

    for (asset_id, entry) in &expected.skills {
        let mut previously_managed = asset_was_managed(lock, updated, asset_id);
        let mut reattested = false;
        let mut desired_snapshot = inputs.skills.get(asset_id).cloned();
        if desired_snapshot.is_none() {
            for destination_root in ctx.destinations {
                let destination = destination_root.join(&entry.skill);
                if let OriginalState::Directory(hash, _) =
                    inspect_skill_directory_state(&destination, ctx.scope)?
                {
                    if hash == entry.hash {
                        let migrated = legacy_migration
                            && legacy_v2_exact_output_is_named(
                                lock,
                                asset_id,
                                &destination,
                                ctx,
                                "tree",
                                &hash,
                                &entry.hash,
                            )?;
                        previously_managed |= migrated;
                        if !migrated {
                            reattested |= portable_output_proof_is_reattestable(
                                lock,
                                asset_id,
                                &destination,
                                ctx.scope,
                                &ctx.scope_root,
                                "skill-tree",
                                "tree",
                                &hash,
                                &entry.hash,
                                persist_lock,
                            )?;
                            if !reattested {
                                require_output_ownership(
                                    lock,
                                    updated,
                                    asset_id,
                                    &destination,
                                    ctx.scope,
                                    &ctx.scope_root,
                                    "skill-tree",
                                    "tree",
                                    &hash,
                                )?;
                            }
                        }
                        desired_snapshot = Some(skill_snapshot_for_scope(
                            TreeSnapshot::capture_destination(&destination)?,
                            ctx.scope,
                        )?);
                        break;
                    }
                }
            }
        }

        let mut changed = reattested;
        for destination_root in ctx.destinations {
            let destination = destination_root.join(&entry.skill);
            let original = inspect_skill_directory_state(&destination, ctx.scope)?;
            match &original {
                OriginalState::Missing => {
                    let snapshot = desired_snapshot.clone().ok_or_else(|| {
                        err(format!(
                            "remote skill `{}` has no verified source input to install",
                            entry.skill
                        ))
                    })?;
                    if !allow_unattested_missing {
                        require_output_ownership(
                            lock,
                            updated,
                            asset_id,
                            &destination,
                            ctx.scope,
                            &ctx.scope_root,
                            "skill-tree",
                            "tree",
                            &entry.hash,
                        )?;
                    }
                    replacements.push(LockedReplacement {
                        destination: destination.clone(),
                        original: original.clone(),
                        payload: ReplacementPayload::Directory(snapshot),
                    });
                    changed = true;
                }
                OriginalState::Directory(current_hash, _) => {
                    let migrated = legacy_migration
                        && legacy_v2_exact_output_is_named(
                            lock,
                            asset_id,
                            &destination,
                            ctx,
                            "tree",
                            current_hash,
                            &entry.hash,
                        )?;
                    previously_managed |= migrated;
                    if !migrated {
                        let reattestable = portable_output_proof_is_reattestable(
                            lock,
                            asset_id,
                            &destination,
                            ctx.scope,
                            &ctx.scope_root,
                            "skill-tree",
                            "tree",
                            current_hash,
                            &entry.hash,
                            persist_lock,
                        )?;
                        if !reattestable {
                            require_output_ownership(
                                lock,
                                updated,
                                asset_id,
                                &destination,
                                ctx.scope,
                                &ctx.scope_root,
                                "skill-tree",
                                "tree",
                                current_hash,
                            )?;
                        }
                        changed |= reattestable;
                    }
                    let needs_mode_normalization = if ctx.scope == Scope::Project {
                        desired_snapshot
                            .as_ref()
                            .map(|snapshot| {
                                TreeSnapshot::capture_destination(&destination)
                                    .map(|current| current != *snapshot)
                            })
                            .transpose()?
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if current_hash != &entry.hash || needs_mode_normalization {
                        let snapshot = desired_snapshot.clone().ok_or_else(|| {
                            err(format!(
                                "remote skill `{}` cannot be repaired without verified source input",
                                entry.skill
                            ))
                        })?;
                        replacements.push(LockedReplacement {
                            destination: destination.clone(),
                            original: original.clone(),
                            payload: ReplacementPayload::Directory(snapshot),
                        });
                        changed = true;
                    }
                }
                OriginalState::File(_, _) => unreachable!(),
            }
            let (key, proof) =
                managed_output(asset_id, &destination, "skill-tree", "tree", &entry.hash);
            active_output_keys.insert(key.clone());
            managed.insert(key, proof);
            if let Some((key, proof)) = portable_output_claim(
                asset_id,
                &destination,
                ctx.scope,
                &ctx.scope_root,
                "skill-tree",
                "tree",
                &entry.hash,
            )? {
                active_portable_keys.insert(key.clone());
                expected.installed_outputs.insert(key, proof);
            }
        }
        if changed {
            if previously_managed {
                result.summary.updated += 1;
            } else {
                result.summary.installed += 1;
            }
            updated_ids.push(asset_id.clone());
        } else {
            result.summary.unchanged += 1;
        }
        result.actions.push(Action {
            source: Some(entry.source.clone()),
            skill: Some(entry.skill.clone()),
            status: if changed {
                if previously_managed {
                    if ctx.dry_run {
                        "would_update"
                    } else {
                        "updated"
                    }
                } else if ctx.dry_run {
                    "would_install"
                } else {
                    "installed"
                }
            } else {
                "unchanged"
            }
            .into(),
            error: None,
        });
    }

    let command_targets = resolve_command_targets(ctx.cfg, ctx.scope, ctx.cfg_dir)?;
    for (asset_id, entry) in expected
        .assets
        .iter()
        .filter(|(_, entry)| entry.kind == "command")
    {
        let mut previously_managed = asset_was_managed(lock, updated, asset_id);
        let source_text = inputs.commands.get(asset_id);
        let mut changed = false;
        for target in &command_targets {
            validate_command_name(&entry.name)?;
            let destination = destination_path(target, &entry.name);
            let format = command_format_binding(target.format);
            let original = inspect_file_state(&destination)?;
            let desired_bytes = if let Some(source_text) = source_text {
                let parsed = parse_command(source_text)?;
                Some(render_command(&parsed, target.format).into_bytes())
            } else {
                None
            };
            match &original {
                OriginalState::Missing => {
                    let bytes = desired_bytes.clone().ok_or_else(|| {
                        err(format!(
                            "remote command `{}` is missing and has no verified source input",
                            entry.name
                        ))
                    })?;
                    let desired_hash = hash_bytes(&bytes);
                    if !allow_unattested_missing {
                        require_output_ownership(
                            lock,
                            updated,
                            asset_id,
                            &destination,
                            ctx.scope,
                            &ctx.scope_root,
                            format,
                            "file",
                            &desired_hash,
                        )?;
                    }
                    replacements.push(LockedReplacement {
                        destination: destination.clone(),
                        original: original.clone(),
                        payload: ReplacementPayload::File {
                            bytes,
                            new_mode: 0o644,
                        },
                    });
                    changed = true;
                }
                OriginalState::File(current_hash, _) => {
                    let desired_hash = desired_bytes.as_ref().map(|bytes| hash_bytes(bytes));
                    let migrated = if let Some(desired_hash) = &desired_hash {
                        legacy_migration
                            && legacy_v2_exact_output_is_named(
                                lock,
                                asset_id,
                                &destination,
                                ctx,
                                "file",
                                current_hash,
                                desired_hash,
                            )?
                    } else {
                        false
                    };
                    previously_managed |= migrated;
                    if !migrated {
                        require_output_ownership(
                            lock,
                            updated,
                            asset_id,
                            &destination,
                            ctx.scope,
                            &ctx.scope_root,
                            format,
                            "file",
                            current_hash,
                        )?;
                    }
                    if let Some(bytes) = &desired_bytes {
                        let desired_hash = hash_bytes(bytes);
                        if current_hash != &desired_hash {
                            replacements.push(LockedReplacement {
                                destination: destination.clone(),
                                original: original.clone(),
                                payload: ReplacementPayload::File {
                                    bytes: bytes.clone(),
                                    new_mode: 0o644,
                                },
                            });
                            changed = true;
                        }
                    }
                }
                OriginalState::Directory(_, _) => unreachable!(),
            }
            let final_hash = desired_bytes
                .as_ref()
                .map(|bytes| hash_bytes(bytes))
                .or_else(|| match &original {
                    OriginalState::File(hash, _) => Some(hash.clone()),
                    _ => None,
                })
                .ok_or_else(|| err("command output has no final hash"))?;
            let (key, proof) = managed_output(asset_id, &destination, format, "file", &final_hash);
            active_output_keys.insert(key.clone());
            managed.insert(key, proof);
            if let Some((key, proof)) = portable_output_claim(
                asset_id,
                &destination,
                ctx.scope,
                &ctx.scope_root,
                format,
                "file",
                &final_hash,
            )? {
                active_portable_keys.insert(key.clone());
                expected.installed_outputs.insert(key, proof);
            }
        }
        if changed {
            if previously_managed {
                result.summary.updated += 1;
            } else {
                result.summary.installed += 1;
            }
            updated_ids.push(asset_id.clone());
        } else {
            result.summary.unchanged += 1;
        }
        result.actions.push(Action {
            source: Some(entry.source.clone()),
            skill: Some(command_action_label(&entry.name)),
            status: if changed {
                if previously_managed {
                    if ctx.dry_run {
                        "would_update"
                    } else {
                        "updated"
                    }
                } else if ctx.dry_run {
                    "would_install"
                } else {
                    "installed"
                }
            } else {
                "unchanged"
            }
            .into(),
            error: None,
        });
    }

    let mcp_targets = resolve_mcp_settings_targets(ctx.cfg, ctx.scope, ctx.cfg_dir)?;
    let mcp_assets = expected
        .assets
        .iter()
        .filter(|(_, entry)| entry.kind == "mcp")
        .collect::<Vec<_>>();
    let mut mcp_changed_assets = HashSet::new();
    let mut legacy_live_mcp_assets = HashSet::new();
    let mut server_claims = HashMap::<(PathBuf, String), String>::new();
    for target in &mcp_targets {
        let original = inspect_file_state(&target.path)?;
        let current_bytes = match &original {
            OriginalState::Missing => None,
            OriginalState::File(_, _) => Some(fs::read(&target.path)?),
            OriginalState::Directory(_, _) => unreachable!(),
        };
        let mut working = current_bytes.clone();

        for (asset_id, entry) in &mcp_assets {
            let format = mcp_format_binding(target.format);
            let existing_proofs =
                historical_mcp_proofs(lock, updated, ctx, asset_id, target, format)?;
            if !existing_proofs.is_empty() {
                let names = existing_proofs
                    .iter()
                    .map(|proof| proof.unit.clone())
                    .collect::<Vec<_>>();
                let current = current_mcp_fragment_hashes(&names, target)?;
                for proof in &existing_proofs {
                    if let Some(actual) = current.get(&proof.unit).and_then(Clone::clone) {
                        if actual != proof.hash {
                            return Err(err(format!(
                                "managed MCP ownership drift for `{}` at {}",
                                proof.unit,
                                target.path.display()
                            )));
                        }
                    }
                }
            }

            if let Some(source_bytes) = inputs.mcps.get(*asset_id) {
                let desired_hashes = expected_mcp_fragment_hashes(source_bytes, target.format)?;
                let desired_names = desired_hashes.keys().cloned().collect::<Vec<_>>();
                let current = current_mcp_fragment_hashes(&desired_names, target)?;
                for name in &desired_names {
                    let claim = (target.path.clone(), name.clone());
                    if let Some(previous) = server_claims.insert(claim, (*asset_id).clone()) {
                        return Err(err(format!(
                            "MCP server `{name}` at {} is claimed by both `{previous}` and `{asset_id}`",
                            target.path.display()
                        )));
                    }
                    if let Some(current_hash) = current.get(name).and_then(Clone::clone) {
                        let migrated = legacy_migration
                            && legacy_v2_exact_output_is_named(
                                lock,
                                asset_id,
                                &target.path,
                                ctx,
                                name,
                                &current_hash,
                                &desired_hashes[name],
                            )?;
                        if migrated {
                            legacy_live_mcp_assets.insert((*asset_id).clone());
                        }
                        if !migrated {
                            require_output_ownership(
                                lock,
                                updated,
                                asset_id,
                                &target.path,
                                ctx.scope,
                                &ctx.scope_root,
                                format,
                                name,
                                &current_hash,
                            )?;
                        }
                    } else if !allow_unattested_missing {
                        require_output_ownership(
                            lock,
                            updated,
                            asset_id,
                            &target.path,
                            ctx.scope,
                            &ctx.scope_root,
                            format,
                            name,
                            &desired_hashes[name],
                        )?;
                    }
                }
                let remove_owned = existing_proofs
                    .iter()
                    .map(|proof| proof.unit.clone())
                    .collect::<Vec<_>>();
                working = Some(render_mcp_settings_bytes(
                    source_bytes,
                    working.as_deref(),
                    target.format,
                    &remove_owned,
                )?);
                replace_managed_target(&mut managed, asset_id, &target.path, format);
                for (name, hash) in desired_hashes {
                    let (key, proof) = managed_output(asset_id, &target.path, format, &name, &hash);
                    active_output_keys.insert(key.clone());
                    managed.insert(key, proof);
                    if let Some((key, proof)) = portable_output_claim(
                        asset_id,
                        &target.path,
                        ctx.scope,
                        &ctx.scope_root,
                        format,
                        &name,
                        &hash,
                    )? {
                        active_portable_keys.insert(key.clone());
                        expected.installed_outputs.insert(key, proof);
                    }
                }
            } else {
                let desired_names = decode_asset_list(&entry.destination, expected.version)?;
                let current = current_mcp_fragment_hashes(&desired_names, target)?;
                for name in desired_names {
                    let claim = (target.path.clone(), name.clone());
                    if let Some(previous) = server_claims.insert(claim, (*asset_id).clone()) {
                        return Err(err(format!(
                            "MCP server `{name}` at {} is claimed by both `{previous}` and `{asset_id}`",
                            target.path.display()
                        )));
                    }
                    let current_hash =
                        current.get(&name).and_then(Clone::clone).ok_or_else(|| {
                            err(format!(
                                "remote MCP server `{name}` is missing with no verified source input at {}",
                                target.path.display()
                            ))
                        })?;
                    require_output_ownership(
                        lock,
                        updated,
                        asset_id,
                        &target.path,
                        ctx.scope,
                        &ctx.scope_root,
                        format,
                        &name,
                        &current_hash,
                    )?;
                    let (key, proof) =
                        managed_output(asset_id, &target.path, format, &name, &current_hash);
                    active_output_keys.insert(key.clone());
                    managed.insert(key, proof);
                    if let Some((key, proof)) = portable_output_claim(
                        asset_id,
                        &target.path,
                        ctx.scope,
                        &ctx.scope_root,
                        format,
                        &name,
                        &current_hash,
                    )? {
                        active_portable_keys.insert(key.clone());
                        expected.installed_outputs.insert(key, proof);
                    }
                }
            }
        }

        // Remove historical MCP units that are no longer active at this target. Project scope
        // derives ownership from portable lock tombstones (so a clean clone works without XDG
        // cache); global scope derives it from the machine-local runtime ledger. Missing units
        // are idempotently accepted, while differing foreign content is never removed.
        let format = mcp_format_binding(target.format);
        let mut stale = BTreeMap::<(String, String), (String, String)>::new();
        if ctx.scope == Scope::Project {
            for (key, proof) in &lock.installed_outputs {
                if proof.format == format
                    && resolve_dest(&proof.destination, &ctx.scope_root) == target.path
                    && !active_portable_keys.contains(key)
                {
                    stale.insert(
                        (proof.asset_id.clone(), proof.unit.clone()),
                        (proof.hash.clone(), key.clone()),
                    );
                }
            }
        } else {
            for (key, proof) in &updated.managed_outputs {
                if proof.destination == target.path.to_string_lossy()
                    && proof.format == format
                    && !active_output_keys.contains(key)
                {
                    if managed_output_key(
                        &proof.asset_id,
                        &proof.destination,
                        &proof.format,
                        &proof.unit,
                    ) != *key
                    {
                        return Err(err(format!(
                            "invalid global managed MCP proof identity at {}",
                            target.path.display()
                        )));
                    }
                    stale.insert(
                        (proof.asset_id.clone(), proof.unit.clone()),
                        (proof.hash.clone(), key.clone()),
                    );
                }
            }
        }
        let stale_names = stale
            .keys()
            .map(|(_, unit)| unit.clone())
            .collect::<Vec<_>>();
        let current_stale = current_mcp_fragment_hashes(&stale_names, target)?;
        let mut present_stale = Vec::new();
        for ((asset_id, unit), (expected_hash, proof_key)) in stale {
            removed_proof_assets.insert(asset_id.clone());
            if let Some(current_hash) = current_stale.get(&unit).and_then(Clone::clone) {
                if current_hash != expected_hash {
                    return Err(err(format!(
                        "refusing to remove drifted managed MCP `{unit}` at {}",
                        target.path.display()
                    )));
                }
                present_stale.push(unit.clone());
            }
            let absolute = target.path.to_string_lossy();
            let runtime_key = managed_output_key(&asset_id, &absolute, format, &unit);
            managed.remove(&runtime_key);
            if persist_lock {
                expected.installed_outputs.remove(&proof_key);
            }
        }
        if !present_stale.is_empty() {
            working = Some(render_mcp_settings_bytes(
                br#"{"mcpServers":{}}"#,
                working.as_deref(),
                target.format,
                &present_stale,
            )?);
        }

        if working != current_bytes {
            replacements.push(LockedReplacement {
                destination: target.path.clone(),
                original,
                payload: ReplacementPayload::File {
                    bytes: working.unwrap_or_default(),
                    new_mode: if ctx.scope == Scope::Global {
                        0o600
                    } else {
                        0o644
                    },
                },
            });
            for (asset_id, _) in &mcp_assets {
                mcp_changed_assets.insert((*asset_id).clone());
            }
        }
    }

    if ctx.scope == Scope::Project {
        let active_targets = mcp_targets
            .iter()
            .map(|target| {
                Ok((
                    relative_project_destination(&target.path, &ctx.scope_root)?
                        .ok_or_else(|| err("project MCP target is outside the project root"))?,
                    mcp_format_binding(target.format).to_string(),
                ))
            })
            .collect::<Result<HashSet<_>>>()?;
        let mut retired = BTreeMap::<(PathBuf, String), Vec<(String, InstalledOutput)>>::new();
        for (key, proof) in &lock.installed_outputs {
            if !is_mcp_output_format(&proof.format)
                || active_portable_keys.contains(key)
                || active_targets.contains(&(proof.destination.clone(), proof.format.clone()))
            {
                continue;
            }
            let destination = resolve_dest(&proof.destination, &ctx.scope_root);
            validate_managed_destination(&destination, ctx.scope, &ctx.scope_root)?;
            retired
                .entry((destination, proof.format.clone()))
                .or_default()
                .push((key.clone(), proof.clone()));
        }
        for ((destination, format), proofs) in retired {
            let target = McpSettingsTarget {
                path: destination.clone(),
                format: parse_mcp_output_format(&format)?,
            };
            let original = inspect_file_state(&destination)?;
            let names = proofs
                .iter()
                .map(|(_, proof)| proof.unit.clone())
                .collect::<Vec<_>>();
            let current = current_mcp_fragment_hashes(&names, &target)?;
            let mut present = Vec::new();
            for (key, proof) in proofs {
                removed_proof_assets.insert(proof.asset_id.clone());
                if let Some(hash) = current.get(&proof.unit).and_then(Clone::clone) {
                    if hash != proof.hash {
                        return Err(err(format!(
                            "refusing to remove drifted retired MCP `{}` at {}",
                            proof.unit,
                            destination.display()
                        )));
                    }
                    present.push(proof.unit.clone());
                }
                let absolute = destination.to_string_lossy();
                managed.remove(&managed_output_key(
                    &proof.asset_id,
                    &absolute,
                    &proof.format,
                    &proof.unit,
                ));
                if persist_lock {
                    expected.installed_outputs.remove(&key);
                }
            }
            if !present.is_empty() {
                let current_bytes = fs::read(&destination)?;
                let rendered = render_mcp_settings_bytes(
                    br#"{"mcpServers":{}}"#,
                    Some(&current_bytes),
                    target.format,
                    &present,
                )?;
                replacements.push(LockedReplacement {
                    destination,
                    original,
                    payload: ReplacementPayload::File {
                        bytes: rendered,
                        new_mode: 0o644,
                    },
                });
            }
        }
    } else {
        let home = dirs_home()?;
        let config = dirs_agent_env_config()?;
        let allowed = all_mcp_settings_targets(&home, &config)
            .into_iter()
            .map(|target| {
                (
                    target.path,
                    mcp_format_binding(target.format).to_string(),
                    target.format,
                )
            })
            .collect::<Vec<_>>();
        let current_targets = mcp_targets
            .iter()
            .map(|target| {
                (
                    target.path.clone(),
                    mcp_format_binding(target.format).to_string(),
                )
            })
            .collect::<HashSet<_>>();
        let mut retired = BTreeMap::<(PathBuf, String), Vec<(String, ManagedOutput)>>::new();
        for (key, proof) in &updated.managed_outputs {
            if !is_mcp_output_format(&proof.format) || active_output_keys.contains(key) {
                continue;
            }
            validate_runtime_proof_key(key, proof)?;
            let destination = PathBuf::from(&proof.destination);
            if current_targets.contains(&(destination.clone(), proof.format.clone())) {
                continue;
            }
            if !allowed
                .iter()
                .any(|(path, format, _)| path == &destination && format == &proof.format)
            {
                return Err(err(format!(
                    "refusing global MCP proof outside the supported target set: {}",
                    destination.display()
                )));
            }
            retired
                .entry((destination, proof.format.clone()))
                .or_default()
                .push((key.clone(), proof.clone()));
        }
        for ((destination, format), proofs) in retired {
            let target_format = allowed
                .iter()
                .find(|(path, binding, _)| path == &destination && binding == &format)
                .map(|(_, _, format)| *format)
                .ok_or_else(|| err("retired global MCP target format is unsupported"))?;
            let target = McpSettingsTarget {
                path: destination.clone(),
                format: target_format,
            };
            let original = inspect_file_state(&destination)?;
            let names = proofs
                .iter()
                .map(|(_, proof)| proof.unit.clone())
                .collect::<Vec<_>>();
            let current = current_mcp_fragment_hashes(&names, &target)?;
            let mut present = Vec::new();
            for (key, proof) in proofs {
                removed_proof_assets.insert(proof.asset_id.clone());
                if let Some(hash) = current.get(&proof.unit).and_then(Clone::clone) {
                    if hash != proof.hash {
                        return Err(err(format!(
                            "refusing to remove drifted retired global MCP `{}` at {}",
                            proof.unit,
                            destination.display()
                        )));
                    }
                    present.push(proof.unit.clone());
                }
                managed.remove(&key);
            }
            if !present.is_empty() {
                let current_bytes = fs::read(&destination)?;
                let rendered = render_mcp_settings_bytes(
                    br#"{"mcpServers":{}}"#,
                    Some(&current_bytes),
                    target.format,
                    &present,
                )?;
                replacements.push(LockedReplacement {
                    destination,
                    original,
                    payload: ReplacementPayload::File {
                        bytes: rendered,
                        new_mode: 0o600,
                    },
                });
            }
        }
    }

    for (asset_id, entry) in &mcp_assets {
        let changed = mcp_changed_assets.contains(*asset_id);
        let previously_managed = asset_was_managed(lock, updated, asset_id)
            || legacy_live_mcp_assets.contains(*asset_id);
        if changed {
            if previously_managed {
                result.summary.updated += 1;
            } else {
                result.summary.installed += 1;
            }
            updated_ids.push((*asset_id).clone());
        } else {
            result.summary.unchanged += 1;
        }
        result.actions.push(Action {
            source: Some(entry.source.clone()),
            skill: Some(mcp_action_label(&entry.name)),
            status: if changed {
                if previously_managed {
                    if ctx.dry_run {
                        "would_update"
                    } else {
                        "updated"
                    }
                } else if ctx.dry_run {
                    "would_install"
                } else {
                    "installed"
                }
            } else {
                "unchanged"
            }
            .into(),
            error: None,
        });
    }

    // Project stale removals derive solely from the portable lock root of trust. A disposable
    // runtime cache is never allowed to nominate an arbitrary deletion path. MCP residuals were
    // handled by the staged per-target merge above.
    let stale_portable = if ctx.scope == Scope::Project {
        lock.installed_outputs
            .iter()
            .filter(|(key, proof)| {
                !active_portable_keys.contains(*key) && !is_mcp_output_format(&proof.format)
            })
            .map(|(key, proof)| (key.clone(), proof.clone()))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    for (key, proof) in stale_portable {
        removed_proof_assets.insert(proof.asset_id.clone());
        let destination = resolve_dest(&proof.destination, &ctx.scope_root);
        validate_managed_destination(&destination, ctx.scope, &ctx.scope_root)?;
        let original = if proof.format == "skill-tree" {
            inspect_skill_directory_state(&destination, ctx.scope)?
        } else {
            inspect_file_state(&destination)?
        };
        let current_hash = match &original {
            OriginalState::Directory(hash, _) | OriginalState::File(hash, _) => hash,
            OriginalState::Missing => {
                let absolute = destination.to_string_lossy();
                managed.remove(&managed_output_key(
                    &proof.asset_id,
                    &absolute,
                    &proof.format,
                    &proof.unit,
                ));
                if persist_lock {
                    expected.installed_outputs.remove(&key);
                }
                continue;
            }
        };
        if current_hash != &proof.hash {
            return Err(err(format!(
                "refusing to remove drifted managed output at {}",
                destination.display()
            )));
        }
        replacements.push(LockedReplacement {
            destination: destination.clone(),
            original,
            payload: ReplacementPayload::Delete,
        });
        let absolute = destination.to_string_lossy();
        managed.remove(&managed_output_key(
            &proof.asset_id,
            &absolute,
            &proof.format,
            &proof.unit,
        ));
        if persist_lock {
            expected.installed_outputs.remove(&key);
        }
    }
    if ctx.scope == Scope::Global {
        let stale_runtime = updated
            .managed_outputs
            .iter()
            .filter(|(key, proof)| {
                !active_output_keys.contains(*key) && !is_mcp_output_format(&proof.format)
            })
            .map(|(key, proof)| (key.clone(), proof.clone()))
            .collect::<Vec<_>>();
        for (key, proof) in stale_runtime {
            removed_proof_assets.insert(proof.asset_id.clone());
            validate_runtime_proof_key(&key, &proof)?;
            if !global_runtime_output_allowed(&proof)? {
                return Err(err(format!(
                    "refusing global managed-output proof outside the supported target set: {}",
                    proof.destination
                )));
            }
            let destination = PathBuf::from(&proof.destination);
            validate_managed_destination(&destination, ctx.scope, &ctx.scope_root)?;
            let original = if proof.format == "skill-tree" {
                inspect_directory_state(&destination)?
            } else {
                inspect_file_state(&destination)?
            };
            let current_hash = match &original {
                OriginalState::Directory(hash, _) | OriginalState::File(hash, _) => hash,
                OriginalState::Missing => {
                    managed.remove(&key);
                    continue;
                }
            };
            if current_hash != &proof.hash {
                return Err(err(format!(
                    "refusing to remove drifted global managed output at {}",
                    destination.display()
                )));
            }
            replacements.push(LockedReplacement {
                destination,
                original,
                payload: ReplacementPayload::Delete,
            });
            managed.remove(&key);
        }
    }

    if ctx.scope == Scope::Project {
        managed.retain(|key, _| active_output_keys.contains(key));
    }

    let mut removed_proof_assets = removed_proof_assets.into_iter().collect::<Vec<_>>();
    removed_proof_assets.sort();
    for asset_id in removed_proof_assets {
        // A retired output target for an otherwise-active asset is an update to that asset, not
        // an asset removal. Count only identities absent from the resulting desired snapshot.
        if expected.skills.contains_key(&asset_id) || expected.assets.contains_key(&asset_id) {
            continue;
        }
        let (source, label) =
            if let Some((source, name)) = parse_framed_asset_id("skill", &asset_id) {
                (source, name.to_string())
            } else if let Some((source, name)) = parse_framed_asset_id("command", &asset_id) {
                (source, command_action_label(name))
            } else if let Some((source, name)) = parse_framed_asset_id("mcp", &asset_id) {
                (source, mcp_action_label(name))
            } else {
                return Err(err(format!(
                    "removed ownership proof has malformed asset identity: {asset_id}"
                )));
            };
        result.summary.removed += 1;
        result.actions.push(Action {
            source: Some(source.to_string()),
            skill: Some(label),
            status: if ctx.dry_run {
                "would_remove"
            } else {
                "removed"
            }
            .into(),
            error: None,
        });
    }

    let transaction_time = now_unix_str();
    let mut final_installed_at = updated.installed_at.clone();
    final_installed_at.retain(|asset_id, _| {
        expected.skills.contains_key(asset_id) || expected.assets.contains_key(asset_id)
    });
    for asset_id in &updated_ids {
        final_installed_at.insert(asset_id.clone(), transaction_time.clone());
    }
    let final_last_run = Some(transaction_time);
    let final_latest_report = Some(serde_json::to_string(&result)?);

    if !ctx.dry_run {
        if persist_lock {
            let lock_bytes = serde_yaml::to_string(&expected)
                .map_err(|error| err(format!("failed to serialize agent lock: {error}")))?
                .into_bytes();
            let destination = lock_path(ctx.scope, ctx.cfg_dir, &dirs_agent_env_data()?);
            let original = inspect_file_state(&destination)?;
            if !matches!(&original, OriginalState::File(hash, _) if hash == &hash_bytes(&lock_bytes))
            {
                replacements.push(LockedReplacement {
                    destination,
                    original,
                    payload: ReplacementPayload::File {
                        bytes: lock_bytes,
                        new_mode: if ctx.scope == Scope::Global {
                            0o600
                        } else {
                            0o644
                        },
                    },
                });
            }
        }

        let runtime = crate::runtime::RuntimeState {
            last_run: final_last_run.clone(),
            latest_report: final_latest_report.clone(),
            installed_at: final_installed_at.clone(),
            managed_outputs: managed.clone(),
        };
        let runtime_bytes = serde_json::to_vec_pretty(&runtime)?;
        let runtime_path = runtime_state_path(ctx.scope, ctx.cfg_dir)?;
        let original = inspect_file_state(&runtime_path)?;
        if !matches!(&original, OriginalState::File(hash, _) if hash == &hash_bytes(&runtime_bytes))
        {
            replacements.push(LockedReplacement {
                destination: runtime_path,
                original,
                payload: ReplacementPayload::File {
                    bytes: runtime_bytes,
                    new_mode: 0o600,
                },
            });
        }
    }

    Ok(LockedSyncPlan {
        result,
        replacements,
        managed_outputs: managed,
        resulting_lock: persist_lock.then_some(expected),
        final_installed_at,
        final_last_run,
        final_latest_report,
    })
}

fn apply_sync_plan(
    ctx: &DriverCtx,
    lock: &mut AgentLockFile,
    updated: &mut UpdatedAt,
    plan: LockedSyncPlan,
) -> SyncResult {
    apply_sync_plan_with_fault(ctx, lock, updated, plan, None)
}

fn apply_sync_plan_with_fault(
    ctx: &DriverCtx,
    lock: &mut AgentLockFile,
    updated: &mut UpdatedAt,
    plan: LockedSyncPlan,
    fault: Option<TransactionFault>,
) -> SyncResult {
    if ctx.dry_run {
        return plan.result;
    }
    if let Err(error) = commit_replacements_inner(&plan.replacements, fault, true) {
        let rollback_coherent = error.rollback_complete;
        let mut failure = sync_preflight_error(
            ctx.locked,
            format!("sync transaction failed: {}", error.error),
        );
        if rollback_coherent {
            failure = persist_coherent_failure_report(ctx, updated, failure);
        }
        return failure;
    }
    if let Some(resulting_lock) = plan.resulting_lock {
        *lock = resulting_lock;
    }
    updated.managed_outputs = plan.managed_outputs;
    updated.installed_at = plan.final_installed_at;
    updated.last_run = plan.final_last_run;
    updated.latest_report = plan.final_latest_report;
    plan.result
}

struct StagedReplacement {
    replacement: LockedReplacement,
    stage: Option<PathBuf>,
    backup: PathBuf,
    rollback_payload: Option<ReplacementPayload>,
}

#[derive(Debug)]
struct TransactionFailure {
    error: crate::AgentEnvError,
    rollback_complete: bool,
}

impl TransactionFailure {
    fn coherent(error: crate::AgentEnvError) -> Self {
        Self {
            error,
            rollback_complete: true,
        }
    }

    fn rollback_failed(error: crate::AgentEnvError) -> Self {
        Self {
            error,
            rollback_complete: false,
        }
    }
}

impl From<crate::AgentEnvError> for TransactionFailure {
    fn from(error: crate::AgentEnvError) -> Self {
        Self::coherent(error)
    }
}

impl From<std::io::Error> for TransactionFailure {
    fn from(error: std::io::Error) -> Self {
        Self::coherent(error.into())
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransactionFault {
    FileStageWrite(usize),
    TreeStageWrite(usize),
    TreeStageCapture(usize),
    BeforeCommit(usize),
    StageRenameAndRestore(usize),
    BackupCleanup(usize),
    BackupCleanupThenRollbackReconstruction {
        cleanup_index: usize,
        reconstruction_index: usize,
    },
}

fn offset_transaction_fault(fault: TransactionFault, offset: usize) -> TransactionFault {
    match fault {
        TransactionFault::FileStageWrite(index) => TransactionFault::FileStageWrite(index + offset),
        TransactionFault::TreeStageWrite(index) => TransactionFault::TreeStageWrite(index + offset),
        TransactionFault::TreeStageCapture(index) => {
            TransactionFault::TreeStageCapture(index + offset)
        }
        TransactionFault::BeforeCommit(index) => TransactionFault::BeforeCommit(index + offset),
        TransactionFault::StageRenameAndRestore(index) => {
            TransactionFault::StageRenameAndRestore(index + offset)
        }
        TransactionFault::BackupCleanup(index) => TransactionFault::BackupCleanup(index + offset),
        TransactionFault::BackupCleanupThenRollbackReconstruction {
            cleanup_index,
            reconstruction_index,
        } => TransactionFault::BackupCleanupThenRollbackReconstruction {
            cleanup_index: cleanup_index + offset,
            reconstruction_index: reconstruction_index + offset,
        },
    }
}

fn commit_replacements_inner(
    replacements: &[LockedReplacement],
    fault: Option<TransactionFault>,
    strict_backup_cleanup: bool,
) -> std::result::Result<(), TransactionFailure> {
    let mut seen = HashSet::new();
    let mut created_parents = Vec::new();
    let mut staged = Vec::new();

    for (stage_index, replacement) in replacements.iter().enumerate() {
        if !seen.insert(replacement.destination.clone()) {
            cleanup_staged(&staged, &created_parents);
            return Err(TransactionFailure::coherent(err(format!(
                "transaction has duplicate destination {}",
                replacement.destination.display()
            ))));
        }
        let Some(parent) = replacement.destination.parent() else {
            cleanup_staged(&staged, &created_parents);
            return Err(TransactionFailure::coherent(err(
                "transaction destination has no parent",
            )));
        };
        if let Err(error) = ensure_parent_recorded(parent, &mut created_parents) {
            cleanup_staged(&staged, &created_parents);
            return Err(TransactionFailure::coherent(error));
        }
        let name = replacement
            .destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("agent-output");
        let suffix = transaction_suffix();
        let stage = parent.join(format!(".{name}.envctl-tx-stage-{suffix}"));
        let backup = parent.join(format!(".{name}.envctl-tx-backup-{suffix}"));
        let rollback_payload = match &replacement.original {
            OriginalState::Missing => None,
            OriginalState::File(_, mode) => match fs::read(&replacement.destination) {
                Ok(bytes) => Some(ReplacementPayload::File {
                    bytes,
                    new_mode: *mode,
                }),
                Err(error) => {
                    cleanup_staged(&staged, &created_parents);
                    return Err(TransactionFailure::coherent(error.into()));
                }
            },
            OriginalState::Directory(_, _) => {
                match TreeSnapshot::capture_destination(&replacement.destination) {
                    Ok(snapshot) => Some(ReplacementPayload::Directory(snapshot)),
                    Err(error) => {
                        cleanup_staged(&staged, &created_parents);
                        return Err(TransactionFailure::coherent(error));
                    }
                }
            }
        };
        let staged_result = match &replacement.payload {
            ReplacementPayload::Directory(snapshot) => snapshot
                .materialize_staged_inner(
                    &stage,
                    match fault {
                        Some(TransactionFault::TreeStageWrite(index)) if index == stage_index => {
                            crate::tree::MaterializeStageFault::AfterCreate
                        }
                        Some(TransactionFault::TreeStageCapture(index)) if index == stage_index => {
                            crate::tree::MaterializeStageFault::BeforeCapture
                        }
                        _ => crate::tree::MaterializeStageFault::None,
                    },
                )
                .map(|()| Some(stage)),
            ReplacementPayload::File { bytes, new_mode } => {
                let mode = match replacement.original {
                    OriginalState::File(_, mode) => mode & *new_mode,
                    _ => *new_mode,
                };
                create_new_staged_file_inner(
                    &stage,
                    bytes,
                    mode,
                    fault == Some(TransactionFault::FileStageWrite(stage_index)),
                )
                .map(|()| Some(stage))
            }
            ReplacementPayload::Delete => Ok(None),
        };
        let stage = match staged_result {
            Ok(stage) => stage,
            Err(error) => {
                cleanup_staged(&staged, &created_parents);
                return Err(TransactionFailure::coherent(error));
            }
        };
        staged.push(StagedReplacement {
            replacement: replacement.clone(),
            stage,
            backup,
            rollback_payload,
        });
    }

    // Revalidate every live input after all outputs are staged and before the first commit.
    for item in &staged {
        if let Some(parent) = item.replacement.destination.parent() {
            if let Err(error) = validate_transaction_parent_chain(parent) {
                cleanup_staged(&staged, &created_parents);
                return Err(TransactionFailure::coherent(error));
            }
        }
        let current = match inspect_original_like(
            &item.replacement.destination,
            &item.replacement.original,
        ) {
            Ok(current) => current,
            Err(error) => {
                cleanup_staged(&staged, &created_parents);
                return Err(TransactionFailure::coherent(error));
            }
        };
        if current != item.replacement.original {
            cleanup_staged(&staged, &created_parents);
            return Err(TransactionFailure::coherent(err(format!(
                "destination changed during locked transaction: {}",
                item.replacement.destination.display()
            ))));
        }
    }

    let mut committed = Vec::<usize>::new();
    for (index, item) in staged.iter().enumerate() {
        let result = (|| -> Result<()> {
            if fault == Some(TransactionFault::BeforeCommit(index)) {
                return Err(err(format!(
                    "injected commit failure at replacement {index}"
                )));
            }
            let existed = !matches!(item.replacement.original, OriginalState::Missing);
            if existed {
                fs::rename(&item.replacement.destination, &item.backup)?;
                sync_parent_directory(&item.replacement.destination)?;
            }
            if let Some(stage) = &item.stage {
                let rename = if matches!(
                    fault,
                    Some(TransactionFault::StageRenameAndRestore(failed)) if failed == index
                ) {
                    Err(std::io::Error::other("injected stage rename failure"))
                } else {
                    fs::rename(stage, &item.replacement.destination)
                };
                if let Err(error) = rename {
                    if existed {
                        let restore =
                            if fault == Some(TransactionFault::StageRenameAndRestore(index)) {
                                Err(std::io::Error::other(
                                    "injected current-item restore failure",
                                ))
                            } else {
                                fs::rename(&item.backup, &item.replacement.destination)
                            };
                        if let Err(restore) = restore {
                            return Err(err(format!(
                                "{error}; current-item restore also failed: {restore}; recovery backup retained at {}",
                                item.backup.display()
                            )));
                        }
                        sync_parent_directory(&item.replacement.destination)?;
                    }
                    return Err(error.into());
                }
                sync_parent_directory(&item.replacement.destination)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            let rollback_error = rollback_committed(&staged, &committed, fault);
            let current_restored =
                inspect_original_like(&item.replacement.destination, &item.replacement.original)
                    .map(|current| current == item.replacement.original)
                    .unwrap_or(false);
            cleanup_staged(&staged, &created_parents);
            return match (rollback_error, current_restored) {
                (Ok(()), true) => Err(TransactionFailure::coherent(error)),
                (Ok(()), false) => Err(TransactionFailure::rollback_failed(error)),
                (Err(rollback), _) => Err(TransactionFailure::rollback_failed(err(format!(
                    "{error}; rollback also failed: {rollback}"
                )))),
            };
        }
        committed.push(index);
    }

    // All commits succeeded. Strict transactions retain in-memory rollback payloads so even a
    // late backup-cleanup failure can reconstruct every earlier original before returning.
    for &index in &committed {
        let backup = &staged[index].backup;
        if fs::symlink_metadata(backup).is_ok() {
            if strict_backup_cleanup {
                let inject_cleanup_failure = fault == Some(TransactionFault::BackupCleanup(index))
                    || matches!(
                        fault,
                        Some(TransactionFault::BackupCleanupThenRollbackReconstruction {
                            cleanup_index,
                            ..
                        }) if cleanup_index == index
                    );
                let cleanup = if inject_cleanup_failure {
                    Err(err(format!(
                        "injected backup cleanup failure; recoverable backup retained at {}",
                        backup.display()
                    )))
                } else {
                    force_remove(backup).map_err(|error| {
                        err(format!(
                            "backup cleanup failed: {error}; recoverable backup retained at {}",
                            backup.display()
                        ))
                    })
                };
                if let Err(error) = cleanup {
                    return match rollback_committed(&staged, &committed, fault) {
                        Ok(()) => {
                            cleanup_staged(&staged, &created_parents);
                            Err(TransactionFailure::coherent(error))
                        }
                        Err(rollback) => Err(TransactionFailure::rollback_failed(err(format!(
                            "{error}; cleanup rollback also failed: {rollback}"
                        )))),
                    };
                }
            } else {
                let _ = force_remove(backup);
            }
        }
    }
    Ok(())
}

fn transaction_suffix() -> String {
    let sequence = TRANSACTION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut random = [0u8; 16];
    let random_ok = fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut random))
        .is_ok();
    if random_ok {
        let hex = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!("{}-{sequence}-{hex}", std::process::id())
    } else {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("{}-{sequence}-{nanos}", std::process::id())
    }
}

fn create_new_staged_file(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    create_new_staged_file_inner(path, bytes, mode, false)
}

fn create_new_staged_file_inner(
    path: &Path,
    bytes: &[u8],
    mode: u32,
    fail_after_create: bool,
) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let result = (|| -> Result<()> {
        if fail_after_create {
            return Err(err(format!(
                "injected staged file write failure: {}",
                path.display()
            )));
        }
        file.write_all(bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(mode))?;
        }
        file.sync_all()?;
        drop(file);
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || fs::read(path)? != bytes {
            return Err(err(format!(
                "staged file verification failed: {}",
                path.display()
            )));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

fn inspect_original_like(path: &Path, original: &OriginalState) -> Result<OriginalState> {
    match original {
        OriginalState::Directory(_, git_portable) => {
            if *git_portable {
                inspect_skill_directory_state(path, Scope::Project)
            } else {
                inspect_directory_state(path)
            }
        }
        OriginalState::File(_, _) => inspect_file_state(path),
        OriginalState::Missing => match fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(OriginalState::Missing)
            }
            Err(error) => Err(error.into()),
            Ok(_) => Err(err(format!(
                "expected destination to remain absent: {}",
                path.display()
            ))),
        },
    }
}

fn rollback_committed(
    staged: &[StagedReplacement],
    committed: &[usize],
    fault: Option<TransactionFault>,
) -> Result<()> {
    let mut failures = Vec::new();
    for index in committed.iter().rev() {
        let item = &staged[*index];
        let parent = item
            .replacement
            .destination
            .parent()
            .ok_or_else(|| err("rollback destination has no parent"))?;
        let name = item
            .replacement
            .destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("agent-output");
        let recovery = parent.join(format!(
            ".{name}.envctl-tx-new-recovery-{}",
            transaction_suffix()
        ));

        if matches!(item.replacement.original, OriginalState::Missing) {
            if fs::symlink_metadata(&item.replacement.destination).is_ok() {
                if let Err(error) = fs::rename(&item.replacement.destination, &recovery)
                    .map_err(crate::AgentEnvError::from)
                    .and_then(|()| sync_parent_directory(&item.replacement.destination))
                    .and_then(|()| force_remove(&recovery))
                {
                    failures.push(format!(
                        "failed to restore absent namespace at {}: {error}; committed output recovery retained at {}",
                        item.replacement.destination.display(),
                        recovery.display()
                    ));
                }
            }
            continue;
        }

        // Prepare and verify the entire prior payload before moving the committed new output.
        // If reconstruction fails, the new output remains at its canonical destination.
        let candidate = if fs::symlink_metadata(&item.backup).is_ok() {
            item.backup.clone()
        } else {
            let reconstruct_fault = matches!(
                fault,
                Some(TransactionFault::BackupCleanupThenRollbackReconstruction {
                    reconstruction_index,
                    ..
                }) if reconstruction_index == *index
            );
            if reconstruct_fault {
                failures.push(format!(
                    "injected rollback reconstruction failure at replacement {}; committed output retained at {}",
                    index,
                    item.replacement.destination.display()
                ));
                continue;
            }
            let candidate = parent.join(format!(
                ".{name}.envctl-tx-old-reconstruction-{}",
                transaction_suffix()
            ));
            let reconstruction = match &item.rollback_payload {
                Some(ReplacementPayload::File { bytes, new_mode }) => {
                    create_new_staged_file(&candidate, bytes, *new_mode)
                }
                Some(ReplacementPayload::Directory(snapshot)) => {
                    snapshot.materialize_staged(&candidate)
                }
                _ => Err(err("transaction rollback payload is missing")),
            };
            if let Err(error) = reconstruction {
                let _ = force_remove(&candidate);
                failures.push(format!(
                    "failed to stage prior state for {}: {error}; committed output retained",
                    item.replacement.destination.display()
                ));
                continue;
            }
            candidate
        };

        // Backups and reconstructed payloads are both untrusted until their complete prior-state
        // identity has been reverified. Never move the committed output away first.
        match inspect_original_like(&candidate, &item.replacement.original) {
            Ok(actual) if actual == item.replacement.original => {}
            Ok(_) => {
                failures.push(format!(
                    "staged prior state failed verification for {}; committed output retained at {}",
                    item.replacement.destination.display(),
                    item.replacement.destination.display()
                ));
                continue;
            }
            Err(error) => {
                failures.push(format!(
                    "could not verify staged prior state for {}: {error}; committed output retained",
                    item.replacement.destination.display()
                ));
                continue;
            }
        }

        let moved_new = fs::symlink_metadata(&item.replacement.destination).is_ok();
        if moved_new {
            if let Err(error) = fs::rename(&item.replacement.destination, &recovery) {
                failures.push(format!(
                    "failed to preserve committed output before rollback at {}: {error}",
                    item.replacement.destination.display()
                ));
                continue;
            }
        }
        if let Err(error) = fs::rename(&candidate, &item.replacement.destination) {
            let restore_new = if moved_new {
                fs::rename(&recovery, &item.replacement.destination)
                    .map_err(crate::AgentEnvError::from)
            } else {
                Ok(())
            };
            failures.push(match restore_new {
                Ok(()) => format!(
                    "failed to install staged prior state at {}: {error}; committed output restored",
                    item.replacement.destination.display()
                ),
                Err(restore) => format!(
                    "failed to install staged prior state at {}: {error}; committed output recovery retained at {} after restore failure: {restore}",
                    item.replacement.destination.display(),
                    recovery.display()
                ),
            });
            continue;
        }
        if let Err(error) = sync_parent_directory(&item.replacement.destination) {
            failures.push(error.to_string());
        }
        if moved_new {
            if let Err(error) = force_remove(&recovery) {
                failures.push(format!(
                    "prior state restored at {}, but committed output recovery cleanup failed at {}: {error}",
                    item.replacement.destination.display(),
                    recovery.display()
                ));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(err(failures.join("; ")))
    }
}
fn cleanup_staged(staged: &[StagedReplacement], created_parents: &[PathBuf]) {
    for item in staged {
        if let Some(stage) = &item.stage {
            if fs::symlink_metadata(stage).is_ok() {
                let _ = force_remove(stage);
            }
        }
        // A backup that remains after an error is the recoverable copy. Never delete it from
        // failure cleanup; successful commits remove backups explicitly above.
    }
    for parent in created_parents.iter().rev() {
        let _ = fs::remove_dir(parent);
    }
}

fn sync_parent_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .ok_or_else(|| err("transaction path has no parent directory"))?;
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn ensure_parent_recorded(parent: &Path, created: &mut Vec<PathBuf>) -> Result<()> {
    validate_transaction_parent_chain(parent)?;
    let mut missing = Vec::new();
    let mut cursor = parent;
    loop {
        match fs::symlink_metadata(cursor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(err(format!(
                        "transaction parent must be a real directory: {}",
                        cursor.display()
                    )));
                }
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(cursor.to_path_buf());
                cursor = cursor
                    .parent()
                    .ok_or_else(|| err("no existing ancestor for transaction destination"))?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    for path in missing.into_iter().rev() {
        crate::secure_file::create_private_directory(&path)?;
        created.push(path);
    }
    validate_transaction_parent_chain(parent)?;
    Ok(())
}

fn validate_transaction_parent_chain(parent: &Path) -> Result<()> {
    crate::secure_file::validate_parent_chain(parent)
}

fn force_remove(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
        for entry in fs::read_dir(path)? {
            force_remove(&entry?.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

// ===================================================================================
// Skills (kasetto commands/sync/skills.rs)
// ===================================================================================

fn ensure_locked_satisfiable(
    src: &SourceSpec,
    desired: &[String],
    lock: &AgentLockFile,
) -> Result<()> {
    match &src.skills {
        SkillsField::List(_) => {
            for name in desired {
                let key = skill_key(&src.source, name);
                if !lock.skills.contains_key(&key) {
                    return Err(err(format!(
                        "--locked: skill `{name}` from `{}` is not in the lock",
                        src.source
                    )));
                }
            }
            Ok(())
        }
        SkillsField::Wildcard(_) => {
            let present = lock.skills.values().any(|e| e.source == src.source);
            if present {
                Ok(())
            } else {
                Err(err(format!(
                    "--locked: source `{}` has no entries in the lock",
                    src.source
                )))
            }
        }
    }
}

fn ensure_locked_satisfiable_commands(
    src: &crate::config::CommandSourceSpec,
    desired: &[String],
    lock: &AgentLockFile,
) -> Result<()> {
    match &src.commands {
        CommandsField::List(_) => {
            for name in desired {
                let asset_id = command_asset_id(&src.source, name);
                if lock.get_tracked_asset("command", &asset_id).is_none() {
                    return Err(err(format!(
                        "--locked: command `{name}` from `{}` is not in the lock",
                        src.source
                    )));
                }
            }
            Ok(())
        }
        CommandsField::Wildcard(_) => {
            let present = lock
                .assets
                .values()
                .any(|a| a.kind == "command" && a.source == src.source);
            if present {
                Ok(())
            } else {
                Err(err(format!(
                    "--locked: source `{}` has no command entries in the lock",
                    src.source
                )))
            }
        }
    }
}

fn file_name_str(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn ensure_locked_satisfiable_mcps(
    src: &crate::config::McpSourceSpec,
    desired_file_names: &[String],
    lock: &AgentLockFile,
) -> Result<()> {
    match &src.mcps {
        McpsField::List(_) => {
            for file_name in desired_file_names {
                let asset_id = mcp_asset_id(&src.source, file_name);
                if lock.get_tracked_asset("mcp", &asset_id).is_none() {
                    return Err(err(format!(
                        "--locked: MCP `{file_name}` from `{}` is not in the lock",
                        src.source
                    )));
                }
            }
            Ok(())
        }
        McpsField::Wildcard(_) => {
            let present = lock
                .assets
                .values()
                .any(|a| a.kind == "mcp" && a.source == src.source);
            if present {
                Ok(())
            } else {
                Err(err(format!(
                    "--locked: source `{}` has no MCP entries in the lock",
                    src.source
                )))
            }
        }
    }
}

pub fn rebuild_lock(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
    upgrade_only: &[String],
) -> Result<AgentLockFile> {
    ensure_supported_version(prev.version)?;
    if prev.version != LOCK_VERSION && !upgrade_only.is_empty() {
        return Err(err(format!(
            "agent lock version {} requires a full rebuild before selective upgrades",
            prev.version
        )));
    }
    let destinations = resolve_destinations(cfg_dir, cfg, scope)?;
    validate_declared_ownership(cfg, cfg_dir, scope, prev, &destinations)?;
    let root = scope_root(scope, cfg_dir)?;
    let prev_skills = &prev.skills;

    let upgrade_active = |source_url: &str| -> bool {
        if upgrade_only.is_empty() {
            return true;
        }
        prev_skills
            .values()
            .any(|e| e.source == source_url && upgrade_only.contains(&e.skill))
    };

    let mut new_skills: BTreeMap<String, AgentLockEntry> = BTreeMap::new();
    let mut new_skill_selectors: BTreeMap<String, String> = BTreeMap::new();
    for (i, src) in cfg.skills.iter().enumerate() {
        if !upgrade_active(&src.source) {
            for (id, entry) in prev_skills.iter().filter(|(_, e)| e.source == src.source) {
                new_skills.insert(id.clone(), entry.clone());
                if let Some(selector) = prev.source_selectors.get(id) {
                    new_skill_selectors.insert(id.clone(), selector.clone());
                }
            }
            continue;
        }
        let stage = std::env::temp_dir().join(format!("envctl-agent-lock-{}-{}", now_unix(), i));
        let materialized = materialize_source(src, cfg_dir, &stage)?;
        let select = select_targets(
            &src.skills,
            &materialized.available,
            &materialized.source_root,
        );

        let result = select.and_then(|(targets, broken)| {
            if let Some(b) = broken.first() {
                return Err(err(format!(
                    "skill `{}` not found in {}",
                    b.name, src.source
                )));
            }
            for (name, dir) in targets {
                let id = skill_key(&src.source, &name);
                let hash = skill_snapshot_for_scope(
                    TreeSnapshot::capture_within(&materialized.source_root, &dir)?,
                    scope,
                )?
                .hash();
                let dest = destinations[0].join(&name);
                let (_, description) = read_skill_profile_from_dir(&dir, &name);
                new_skills.insert(
                    id.clone(),
                    AgentLockEntry {
                        destination: relativize_dest(&dest, &root),
                        hash,
                        skill: name.clone(),
                        description,
                        source: src.source.clone(),
                        source_revision: materialized.source_revision.clone(),
                        scope: Some(scope),
                    },
                );
                if let Some(selector) =
                    skill_lock_selector(src, &name, scope, &destinations, &root)?
                {
                    new_skill_selectors.insert(id, selector);
                }
            }
            Ok(())
        });

        if let Some(cleanup) = materialized.cleanup_dir {
            let _ = fs::remove_dir_all(cleanup);
        }
        result?;
    }

    let mut next = prev.clone();
    next.version = LOCK_VERSION;
    for id in prev.skills.keys() {
        next.source_selectors.remove(id);
    }
    next.skills = new_skills;
    next.source_selectors.extend(new_skill_selectors);
    let (new_assets, new_asset_selectors) =
        rebuild_assets(cfg, cfg_dir, scope, prev, upgrade_only, &root)?;
    for id in prev.assets.keys() {
        next.source_selectors.remove(id);
    }
    next.assets = new_assets;
    next.source_selectors.extend(new_asset_selectors);
    Ok(next)
}

fn selector_value(value: Option<&str>) -> String {
    value
        .map(|value| format!("{}:{value}", value.len()))
        .unwrap_or_else(|| "-".into())
}

fn skill_source_selector(src: &SourceSpec, name: &str) -> Result<Option<String>> {
    let selection = match &src.skills {
        SkillsField::Wildcard(value) if value == "*" => "wildcard".into(),
        SkillsField::Wildcard(value) => {
            return Err(err(format!(
                "invalid skills value \"{value}\": expected \"*\" or a list"
            )));
        }
        SkillsField::List(entries) => entries
            .iter()
            .find_map(|entry| match entry {
                SkillTarget::Name(entry_name) if entry_name == name => Some("name".into()),
                SkillTarget::Obj {
                    name: entry_name,
                    path,
                } if entry_name == name => {
                    Some(format!("object:{}", selector_value(path.as_deref())))
                }
                _ => None,
            })
            .ok_or_else(|| err(format!("skill selector for `{name}` is absent")))?,
    };
    Ok(Some(format!(
        "v1|kind=skill|sub-dir={}|selection={selection}",
        selector_value(src.sub_dir.as_deref())
    )))
}

fn scope_binding(scope: Scope) -> &'static str {
    match scope {
        Scope::Project => "project",
        Scope::Global => "global",
    }
}

fn frame_binding(value: &str) -> String {
    format!("{}:{value}", value.len())
}

/// Bind an opaque source selector to its scope and exact destination list without relying on a
/// delimiter search. Both the selector body and every target are byte-length framed, and the
/// target count makes concatenation/trailing-data ambiguity fail closed.
fn bind_selector_targets(base: &str, scope: Scope, targets: &[String]) -> String {
    format!(
        "v2|base={}:{}|scope={}|targets={}|{}",
        base.len(),
        base,
        scope_binding(scope),
        targets.len(),
        targets
            .iter()
            .map(|value| frame_binding(value))
            .collect::<String>()
    )
}

fn skill_lock_selector(
    src: &SourceSpec,
    name: &str,
    scope: Scope,
    destinations: &[PathBuf],
    root: &Path,
) -> Result<Option<String>> {
    let Some(base) = skill_source_selector(src, name)? else {
        return Ok(None);
    };
    let mut bound = destinations
        .iter()
        .map(|destination| relativize_dest(&destination.join(name), root))
        .collect::<Vec<_>>();
    bound.sort();
    Ok(Some(bind_selector_targets(&base, scope, &bound)))
}

fn command_source_selector(
    src: &crate::config::CommandSourceSpec,
    name: &str,
) -> Result<Option<String>> {
    let selection = match &src.commands {
        CommandsField::Wildcard(value) if value == "*" => "wildcard".into(),
        CommandsField::Wildcard(value) => {
            return Err(err(format!(
                "invalid commands value \"{value}\": expected \"*\" or a list"
            )));
        }
        CommandsField::List(entries) => entries
            .iter()
            .find_map(|entry| match entry {
                CommandEntry::Name(entry_name) if entry_name == name => Some("name".into()),
                CommandEntry::Obj {
                    name: entry_name,
                    path,
                } if entry_name == name => {
                    Some(format!("object:{}", selector_value(path.as_deref())))
                }
                _ => None,
            })
            .ok_or_else(|| err(format!("command selector for `{name}` is absent")))?,
    };
    Ok(Some(format!(
        "v1|kind=command|sub-dir={}|selection={selection}",
        selector_value(src.sub_dir.as_deref())
    )))
}

fn command_format_binding(format: crate::agent::CommandFormat) -> &'static str {
    match format {
        crate::agent::CommandFormat::MarkdownFrontmatter => "markdown-frontmatter",
        crate::agent::CommandFormat::MarkdownPlain => "markdown-plain",
        crate::agent::CommandFormat::PromptMd => "prompt-md",
        crate::agent::CommandFormat::PromptFile => "prompt-file",
        crate::agent::CommandFormat::GeminiToml => "gemini-toml",
    }
}

fn command_lock_selector(
    src: &crate::config::CommandSourceSpec,
    name: &str,
    scope: Scope,
    targets: &[CommandTarget],
    root: &Path,
) -> Result<Option<String>> {
    let Some(base) = command_source_selector(src, name)? else {
        return Ok(None);
    };
    let mut bound = targets
        .iter()
        .map(|target| {
            format!(
                "{}@{}",
                relativize_dest(&destination_path(target, name), root),
                command_format_binding(target.format)
            )
        })
        .collect::<Vec<_>>();
    bound.sort();
    Ok(Some(bind_selector_targets(&base, scope, &bound)))
}

fn mcp_source_selector(
    src: &crate::config::McpSourceSpec,
    file_name: &str,
) -> Result<Option<String>> {
    let selection = match &src.mcps {
        McpsField::Wildcard(value) if value == "*" => "wildcard".into(),
        McpsField::Wildcard(value) => {
            return Err(err(format!(
                "invalid mcps value \"{value}\": expected \"*\" or a list"
            )));
        }
        McpsField::List(entries) => entries
            .iter()
            .find_map(|entry| {
                if desired_mcp_file_name_for_entry(entry) != file_name {
                    return None;
                }
                Some(match entry {
                    McpEntry::Name(_) => "name".into(),
                    McpEntry::Obj { path, .. } => {
                        format!("object:{}", selector_value(path.as_deref()))
                    }
                })
            })
            .ok_or_else(|| err(format!("MCP selector for `{file_name}` is absent")))?,
    };
    Ok(Some(format!("v1|kind=mcp|selection={selection}")))
}

fn mcp_format_binding(format: crate::agent::McpSettingsFormat) -> &'static str {
    match format {
        crate::agent::McpSettingsFormat::McpServers => "mcp-servers-json",
        crate::agent::McpSettingsFormat::VsCodeServers => "vscode-servers-json",
        crate::agent::McpSettingsFormat::OpenCode => "opencode-json",
        crate::agent::McpSettingsFormat::CodexToml => "codex-toml",
    }
}

fn is_mcp_output_format(format: &str) -> bool {
    matches!(
        format,
        "mcp-servers-json" | "vscode-servers-json" | "opencode-json" | "codex-toml"
    )
}

fn parse_mcp_output_format(format: &str) -> Result<crate::agent::McpSettingsFormat> {
    match format {
        "mcp-servers-json" => Ok(crate::agent::McpSettingsFormat::McpServers),
        "vscode-servers-json" => Ok(crate::agent::McpSettingsFormat::VsCodeServers),
        "opencode-json" => Ok(crate::agent::McpSettingsFormat::OpenCode),
        "codex-toml" => Ok(crate::agent::McpSettingsFormat::CodexToml),
        _ => Err(err(format!("unknown MCP output format `{format}`"))),
    }
}

fn mcp_lock_selector(
    src: &crate::config::McpSourceSpec,
    file_name: &str,
    scope: Scope,
    targets: &[McpSettingsTarget],
    root: &Path,
) -> Result<Option<String>> {
    let Some(base) = mcp_source_selector(src, file_name)? else {
        return Ok(None);
    };
    let mut bound = targets
        .iter()
        .map(|target| {
            format!(
                "{}@{}",
                relativize_dest(&target.path, root),
                mcp_format_binding(target.format)
            )
        })
        .collect::<Vec<_>>();
    bound.sort();
    Ok(Some(bind_selector_targets(&base, scope, &bound)))
}

fn rebuild_assets(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
    upgrade_only: &[String],
    root: &Path,
) -> Result<(BTreeMap<String, AssetEntry>, BTreeMap<String, String>)> {
    let mut assets: BTreeMap<String, AssetEntry> = prev
        .assets
        .iter()
        .filter(|(_, entry)| entry.kind != "command" && entry.kind != "mcp")
        .map(|(id, entry)| (id.clone(), entry.clone()))
        .collect();
    let mut selectors: BTreeMap<String, String> = assets
        .keys()
        .filter_map(|id| {
            prev.source_selectors
                .get(id)
                .map(|selector| (id.clone(), selector.clone()))
        })
        .collect();

    let command_targets = resolve_command_targets(cfg, scope, cfg_dir)?;
    let mcp_targets = resolve_mcp_settings_targets(cfg, scope, cfg_dir)?;
    if !cfg.commands.is_empty() && command_targets.is_empty() {
        return Err(err(
            "agent lock resolved no command destination for configured commands",
        ));
    }
    for (index, src) in cfg.commands.iter().enumerate() {
        let desired = desired_command_names(src, prev);
        let rebuild = upgrade_only.is_empty()
            || desired.iter().any(|name| upgrade_only.contains(name))
            || prev.assets.values().any(|entry| {
                entry.kind == "command"
                    && entry.source == src.source
                    && upgrade_only.contains(&entry.name)
            });
        if !rebuild {
            carry_locked_assets("command", &src.source, prev, &mut assets, &mut selectors);
            continue;
        }

        let stage =
            std::env::temp_dir().join(format!("envctl-agent-lock-command-{}-{index}", now_unix()));
        let materialized = materialize_source(&src.as_source_spec(), cfg_dir, &stage)?;
        let result = (|| {
            let selected = select_commands_for_audit(src, &materialized.source_root)?;
            if selected.is_empty()
                && matches!(&src.commands, CommandsField::Wildcard(value) if value == "*")
            {
                return Err(err(format!(
                    "no commands found in source `{}` (expected commands/*.md)",
                    src.source
                )));
            }
            for (name, path) in selected {
                validate_command_name(&name)?;
                let source_bytes =
                    TreeSnapshot::capture_file_within(&materialized.source_root, &path)?;
                let source_text = String::from_utf8(source_bytes.clone())
                    .map_err(|error| err(format!("command source is not UTF-8: {error}")))?;
                parse_command(&source_text)?;
                let id = command_asset_id(&src.source, &name);
                let destination = encode_asset_list(
                    command_targets
                        .iter()
                        .map(|target| relativize_dest(&destination_path(target, &name), root)),
                );
                assets.insert(
                    id.clone(),
                    AssetEntry {
                        kind: "command".into(),
                        name: name.clone(),
                        hash: hash_bytes(&source_bytes),
                        source: src.source.clone(),
                        destination,
                        source_revision: materialized.source_revision.clone(),
                    },
                );
                if let Some(selector) =
                    command_lock_selector(src, &name, scope, &command_targets, root)?
                {
                    selectors.insert(id, selector);
                }
            }
            Ok(())
        })();
        if let Some(cleanup) = materialized.cleanup_dir {
            let _ = fs::remove_dir_all(cleanup);
        }
        result?;
    }

    for (index, src) in cfg.mcps.iter().enumerate() {
        let desired = desired_mcp_file_names(src, prev);
        let rebuild = upgrade_only.is_empty()
            || desired.iter().any(|name| {
                upgrade_only.contains(name)
                    || name
                        .strip_suffix(".json")
                        .is_some_and(|plain| upgrade_only.iter().any(|item| item == plain))
            })
            || prev.assets.values().any(|entry| {
                entry.kind == "mcp"
                    && entry.source == src.source
                    && (upgrade_only.contains(&entry.name)
                        || entry
                            .name
                            .strip_suffix(".json")
                            .is_some_and(|plain| upgrade_only.iter().any(|item| item == plain)))
            });
        if !rebuild {
            carry_locked_assets("mcp", &src.source, prev, &mut assets, &mut selectors);
            continue;
        }

        let stage =
            std::env::temp_dir().join(format!("envctl-agent-lock-mcp-{}-{index}", now_unix()));
        let materialized = materialize_source(&src.as_source_spec(), cfg_dir, &stage)?;
        let result = (|| {
            let paths = select_mcps_for_audit(src, &materialized.source_root)?;
            if paths.is_empty() {
                return Err(err(format!(
                    "no MCP JSON files found in source `{}`",
                    src.source
                )));
            }
            for path in paths {
                let file_name = file_name_str(&path);
                validate_safe_segment("MCP pack file", &file_name)?;
                let source_bytes =
                    TreeSnapshot::capture_file_within(&materialized.source_root, &path)?;
                let destination = encode_asset_list(mcp_server_names_from_bytes(&source_bytes)?);
                let id = mcp_asset_id(&src.source, &file_name);
                assets.insert(
                    id.clone(),
                    AssetEntry {
                        kind: "mcp".into(),
                        name: file_name.clone(),
                        hash: hash_bytes(&source_bytes),
                        source: src.source.clone(),
                        destination,
                        source_revision: materialized.source_revision.clone(),
                    },
                );
                if let Some(selector) =
                    mcp_lock_selector(src, &file_name, scope, &mcp_targets, root)?
                {
                    selectors.insert(id, selector);
                }
            }
            Ok(())
        })();
        if let Some(cleanup) = materialized.cleanup_dir {
            let _ = fs::remove_dir_all(cleanup);
        }
        result?;
    }

    Ok((assets, selectors))
}

fn carry_locked_assets(
    kind: &str,
    source: &str,
    prev: &AgentLockFile,
    assets: &mut BTreeMap<String, AssetEntry>,
    selectors: &mut BTreeMap<String, String>,
) {
    for (id, entry) in prev
        .assets
        .iter()
        .filter(|(_, entry)| entry.kind == kind && entry.source == source)
    {
        assets.insert(id.clone(), entry.clone());
        if let Some(selector) = prev.source_selectors.get(id) {
            selectors.insert(id.clone(), selector.clone());
        }
    }
}

/// Build the lock snapshot implied by `cfg` without performing any network I/O.
///
/// Local sources are read and SHA-256 hashed in place. Remote sources are never
/// materialized: their configured identities/selectors must already be satisfied by
/// `prev`, including a non-empty hash and the exact configured revision label. The
/// returned snapshot is suitable for [`AgentLockFile::lock_check`]; this function never
/// writes either the lock or an installation destination.
pub fn audit_lock_zero_network(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
) -> Result<AgentLockFile> {
    build_zero_network_snapshot(cfg, cfg_dir, scope, prev).map(|(lock, _)| lock)
}

#[derive(Default)]
struct LockedInputs {
    skills: HashMap<String, TreeSnapshot>,
    commands: HashMap<String, String>,
    mcps: HashMap<String, Vec<u8>>,
}

fn build_zero_network_snapshot(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
) -> Result<(AgentLockFile, LockedInputs)> {
    let destinations = resolve_destinations(cfg_dir, cfg, scope)?;
    validate_declared_ownership(cfg, cfg_dir, scope, prev, &destinations)?;
    let root = scope_root(scope, cfg_dir)?;
    let mut next = AgentLockFile {
        installed_outputs: prev.installed_outputs.clone(),
        ..AgentLockFile::default()
    };
    // Ownership is historical evidence, not desired state. Zero-network audit preserves the
    // exact prior attestations (including inactive tombstones) and never synthesizes new ones.
    let mut inputs = LockedInputs::default();

    audit_skills_zero_network(
        cfg,
        cfg_dir,
        scope,
        prev,
        &destinations,
        &root,
        &mut next,
        &mut inputs,
    )?;
    audit_commands_zero_network(cfg, cfg_dir, scope, prev, &root, &mut next, &mut inputs)?;
    audit_mcps_zero_network(cfg, cfg_dir, scope, prev, &root, &mut next, &mut inputs)?;
    Ok((next, inputs))
}

// Skill auditing needs both config/source context and the resolved multi-agent destination set.
#[allow(clippy::too_many_arguments)]
fn audit_skills_zero_network(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
    destinations: &[PathBuf],
    root: &Path,
    next: &mut AgentLockFile,
    inputs: &mut LockedInputs,
) -> Result<()> {
    let destination = destinations
        .first()
        .ok_or_else(|| err("agent lock audit resolved no skill destination"))?;

    for (i, src) in cfg.skills.iter().enumerate() {
        if src.source.contains("://") {
            let desired = desired_skill_names(src, prev);
            ensure_locked_satisfiable(src, &desired, prev)?;
            for name in desired {
                let key = skill_key(&src.source, &name);
                let locked = prev
                    .skills
                    .get(&key)
                    .ok_or_else(|| err(format!("--locked: skill `{name}` is missing from lock")))?;
                require_remote_skill_pin(src, &name, locked)?;
                let source_selector = skill_lock_selector(src, &name, scope, destinations, root)?;
                let mut expected = locked.clone();
                expected.destination = relativize_dest(&destination.join(&name), root);
                expected.skill = name;
                expected.source = src.source.clone();
                expected.source_revision = locked.source_revision.clone();
                expected.scope = Some(scope);
                next.skills.insert(key.clone(), expected);
                next.set_source_selector(&key, source_selector);
            }
            continue;
        }

        let stage =
            std::env::temp_dir().join(format!("envctl-agent-lock-audit-local-{}-{i}", now_unix()));
        let materialized = materialize_source(src, cfg_dir, &stage)?;
        let (targets, broken) = select_targets(
            &src.skills,
            &materialized.available,
            &materialized.source_root,
        )?;
        if let Some(b) = broken.first() {
            return Err(err(format!(
                "skill `{}` not found in {}",
                b.name, src.source
            )));
        }
        for (name, dir) in targets {
            let id = skill_key(&src.source, &name);
            validate_safe_segment("skill", &name)?;
            let snapshot = skill_snapshot_for_scope(
                TreeSnapshot::capture_within(&materialized.source_root, &dir)?,
                scope,
            )?;
            let hash = snapshot.hash();
            let (_, description) = read_skill_profile_from_dir(&dir, &name);
            next.skills.insert(
                id.clone(),
                AgentLockEntry {
                    destination: relativize_dest(&destination.join(&name), root),
                    hash,
                    skill: name.clone(),
                    description,
                    source: src.source.clone(),
                    source_revision: "local".into(),
                    scope: Some(scope),
                },
            );
            next.set_source_selector(
                &id,
                skill_lock_selector(src, &name, scope, destinations, root)?,
            );
            inputs.skills.insert(id, snapshot);
        }
    }
    Ok(())
}

fn require_remote_skill_pin(src: &SourceSpec, name: &str, locked: &AgentLockEntry) -> Result<()> {
    if locked.hash.is_empty() {
        return Err(err(format!(
            "--locked: remote skill `{name}` from `{}` has no content hash",
            src.source
        )));
    }
    if locked.source != src.source || locked.skill != name {
        return Err(err(format!(
            "--locked: remote skill `{name}` does not match its configured source identity"
        )));
    }
    if locked.source_revision.is_empty() || !src.accepts_resolved_revision(&locked.source_revision)
    {
        return Err(err(format!(
            "--locked: remote skill `{name}` from `{}` is not pinned to {}",
            src.source,
            src.revision_expectation()
        )));
    }
    Ok(())
}

fn audit_commands_zero_network(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
    root: &Path,
    next: &mut AgentLockFile,
    inputs: &mut LockedInputs,
) -> Result<()> {
    let targets = resolve_command_targets(cfg, scope, cfg_dir)?;
    if targets.is_empty() {
        return Ok(());
    }

    for (i, src) in cfg.commands.iter().enumerate() {
        let desired = desired_command_names(src, prev);
        if src.source.contains("://") {
            ensure_locked_satisfiable_commands(src, &desired, prev)?;
            let source_spec = src.as_source_spec();
            for name in desired {
                let id = command_asset_id(&src.source, &name);
                let locked = require_remote_asset(prev, &id, "command", &name, &source_spec)?;
                let destination = encode_asset_list(
                    targets
                        .iter()
                        .map(|target| relativize_dest(&destination_path(target, &name), root)),
                );
                let source_selector = command_lock_selector(src, &name, scope, &targets, root)?;
                next.assets.insert(
                    id.clone(),
                    AssetEntry {
                        kind: "command".into(),
                        name,
                        hash: locked.hash.clone(),
                        source: src.source.clone(),
                        destination,
                        source_revision: locked.source_revision.clone(),
                    },
                );
                next.set_source_selector(&id, source_selector);
            }
            continue;
        }

        let stage = std::env::temp_dir().join(format!(
            "envctl-agent-lock-audit-command-{}-{i}",
            now_unix()
        ));
        let materialized = materialize_source(&src.as_source_spec(), cfg_dir, &stage)?;
        let selected = select_commands_for_audit(src, &materialized.source_root)?;
        for (name, path) in selected {
            validate_command_name(&name)?;
            let source_bytes = TreeSnapshot::capture_file_within(&materialized.source_root, &path)?;
            let source_text = String::from_utf8(source_bytes.clone())
                .map_err(|error| err(format!("command source is not UTF-8: {error}")))?;
            parse_command(&source_text)?;
            let destination = encode_asset_list(
                targets
                    .iter()
                    .map(|target| relativize_dest(&destination_path(target, &name), root)),
            );
            let id = command_asset_id(&src.source, &name);
            next.assets.insert(
                id.clone(),
                AssetEntry {
                    kind: "command".into(),
                    name: name.clone(),
                    hash: hash_bytes(&source_bytes),
                    source: src.source.clone(),
                    destination,
                    source_revision: "local".into(),
                },
            );
            next.set_source_selector(
                &id,
                command_lock_selector(src, &name, scope, &targets, root)?,
            );
            inputs.commands.insert(id, source_text);
        }
    }
    Ok(())
}

fn select_commands_for_audit(
    src: &crate::config::CommandSourceSpec,
    root: &Path,
) -> Result<Vec<(String, PathBuf)>> {
    match &src.commands {
        CommandsField::Wildcard(s) if s == "*" => {
            Ok(discover_commands(root)?.into_iter().collect())
        }
        CommandsField::Wildcard(s) => Err(err(format!(
            "invalid commands value \"{s}\": expected \"*\" or a list"
        ))),
        CommandsField::List(entries) => entries
            .iter()
            .map(|entry| resolve_command_entry(root, entry))
            .collect(),
    }
}

fn audit_mcps_zero_network(
    cfg: &Config,
    cfg_dir: &Path,
    scope: Scope,
    prev: &AgentLockFile,
    root: &Path,
    next: &mut AgentLockFile,
    inputs: &mut LockedInputs,
) -> Result<()> {
    let targets = resolve_mcp_settings_targets(cfg, scope, cfg_dir)?;
    for (i, src) in cfg.mcps.iter().enumerate() {
        let desired = desired_mcp_file_names(src, prev);
        if src.source.contains("://") {
            ensure_locked_satisfiable_mcps(src, &desired, prev)?;
            let source_spec = src.as_source_spec();
            for file_name in desired {
                let id = mcp_asset_id(&src.source, &file_name);
                let locked = require_remote_asset(prev, &id, "mcp", &file_name, &source_spec)?;
                let source_selector = mcp_lock_selector(src, &file_name, scope, &targets, root)?;
                next.assets.insert(
                    id.clone(),
                    AssetEntry {
                        kind: "mcp".into(),
                        name: file_name,
                        hash: locked.hash.clone(),
                        source: src.source.clone(),
                        destination: locked.destination.clone(),
                        source_revision: locked.source_revision.clone(),
                    },
                );
                next.set_source_selector(&id, source_selector);
            }
            continue;
        }

        let stage =
            std::env::temp_dir().join(format!("envctl-agent-lock-audit-mcp-{}-{i}", now_unix()));
        let materialized = materialize_source(&src.as_source_spec(), cfg_dir, &stage)?;
        let paths = select_mcps_for_audit(src, &materialized.source_root)?;
        for path in paths {
            let file_name = file_name_str(&path);
            validate_safe_segment("MCP pack file", &file_name)?;
            let source_bytes = TreeSnapshot::capture_file_within(&materialized.source_root, &path)?;
            let destination = encode_asset_list(mcp_server_names_from_bytes(&source_bytes)?);
            let id = mcp_asset_id(&src.source, &file_name);
            next.assets.insert(
                id.clone(),
                AssetEntry {
                    kind: "mcp".into(),
                    name: file_name.clone(),
                    hash: hash_bytes(&source_bytes),
                    source: src.source.clone(),
                    destination,
                    source_revision: "local".into(),
                },
            );
            next.set_source_selector(
                &id,
                mcp_lock_selector(src, &file_name, scope, &targets, root)?,
            );
            inputs.mcps.insert(id, source_bytes);
        }
    }
    Ok(())
}

fn select_mcps_for_audit(src: &crate::config::McpSourceSpec, root: &Path) -> Result<Vec<PathBuf>> {
    match &src.mcps {
        McpsField::Wildcard(s) if s == "*" => discover_mcps(root),
        McpsField::Wildcard(s) => Err(err(format!(
            "invalid mcps value \"{s}\": expected \"*\" or a list"
        ))),
        McpsField::List(entries) => entries
            .iter()
            .map(|entry| resolve_mcp_entry(root, entry))
            .collect(),
    }
}

fn require_remote_asset<'a>(
    prev: &'a AgentLockFile,
    id: &str,
    kind: &str,
    name: &str,
    source_spec: &SourceSpec,
) -> Result<&'a AssetEntry> {
    let source = source_spec.source.as_str();
    let locked = prev
        .assets
        .get(id)
        .ok_or_else(|| err(format!("--locked: {kind} `{name}` is missing from lock")))?;
    if locked.kind != kind || locked.name != name || locked.source != source {
        return Err(err(format!(
            "--locked: {kind} `{name}` does not match its configured source identity"
        )));
    }
    if locked.hash.is_empty() {
        return Err(err(format!(
            "--locked: remote {kind} `{name}` from `{source}` has no content hash"
        )));
    }
    if locked.source_revision.is_empty()
        || !source_spec.accepts_resolved_revision(&locked.source_revision)
    {
        return Err(err(format!(
            "--locked: remote {kind} `{name}` from `{source}` is not pinned to {}",
            source_spec.revision_expectation()
        )));
    }
    Ok(locked)
}

// ===================================================================================
// list (kasetto commands/list.rs)
// ===================================================================================

/// A `list`-view row for an installed non-skill asset (MCP server or command).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetRow {
    pub name: String,
    pub scope: Scope,
    pub pack_file: String,
    pub source: String,
}

/// Proof-derived view of live installed outputs for one scope.
#[derive(Clone, Debug, Default)]
pub struct InstalledInventory {
    pub skills: Vec<InstalledSkill>,
    pub mcps: Vec<AssetRow>,
    pub commands: Vec<AssetRow>,
    pub install_paths: Vec<String>,
    pub issues: Vec<String>,
}

/// Inspect the exact scope proof store and verify every claimed output against live bytes.
/// Desired lock entries without a proof are reported as issues, never as installed inventory.
pub fn inspect_installed_inventory(
    lock: &AgentLockFile,
    runtime: &crate::runtime::RuntimeState,
    scope: Scope,
    project_root: &Path,
    composite_ids: bool,
) -> Result<InstalledInventory> {
    let root = scope_root(scope, project_root)?;
    validate_portable_output_claims(lock, scope, &root, &[])?;
    if scope == Scope::Project {
        let updated = UpdatedAt {
            installed_at: runtime.installed_at.clone(),
            managed_outputs: runtime.managed_outputs.clone(),
            last_run: runtime.last_run.clone(),
            latest_report: runtime.latest_report.clone(),
        };
        validate_historical_custom_skill_claims(lock, &updated, &root, &[])?;
    }
    let updated = UpdatedAt {
        installed_at: runtime.installed_at.clone(),
        managed_outputs: runtime.managed_outputs.clone(),
        last_run: runtime.last_run.clone(),
        latest_report: runtime.latest_report.clone(),
    };
    let proofs = collect_clean_proofs(lock, &updated, scope, &root);
    validate_clean_proofs(&proofs, scope)?;

    let proof_assets = proofs
        .iter()
        .map(|(_, proof)| proof.asset_id.clone())
        .collect::<HashSet<_>>();
    let mut inventory = InstalledInventory::default();
    for desired in lock.skills.keys().chain(lock.assets.keys()) {
        if !proof_assets.contains(desired) {
            inventory.issues.push(format!(
                "desired asset has no installed ownership proof: {desired}"
            ));
        }
    }

    let mut listed_skills = HashSet::new();
    let mut listed_commands = HashSet::new();
    let mut listed_mcps = HashSet::new();
    for (_, proof) in proofs {
        if let Some(issue) = live_proof_issue(&proof, scope)? {
            inventory.issues.push(issue);
            continue;
        }
        let destination = PathBuf::from(&proof.destination);
        inventory.install_paths.push(
            destination
                .parent()
                .unwrap_or(&destination)
                .to_string_lossy()
                .into_owned(),
        );
        if proof.format == "skill-tree" {
            let Some((source, name)) = parse_framed_asset_id("skill", &proof.asset_id) else {
                return Err(err(format!(
                    "installed skill proof has malformed asset id: {}",
                    proof.asset_id
                )));
            };
            if !listed_skills.insert(proof.asset_id.clone()) {
                continue;
            }
            let entry = lock.skills.get(&proof.asset_id);
            let skill_name = entry.map(|entry| entry.skill.as_str()).unwrap_or(name);
            let destination_text = destination.to_string_lossy().into_owned();
            let (profile_name, fallback_description) =
                read_skill_profile(&destination_text, skill_name);
            let description = entry
                .map(|entry| entry.description.trim())
                .filter(|description| !description.is_empty())
                .map(str::to_string)
                .unwrap_or(fallback_description);
            let updated_at = runtime
                .installed_at
                .get(&proof.asset_id)
                .cloned()
                .unwrap_or_default();
            inventory.skills.push(InstalledSkill {
                id: skill_display_id(scope, &proof.asset_id, composite_ids),
                scope,
                name: profile_name,
                description,
                source: source.to_string(),
                skill: skill_name.to_string(),
                destination: destination_text,
                hash: proof.hash.clone(),
                source_revision: entry
                    .map(|entry| entry.source_revision.clone())
                    .unwrap_or_default(),
                updated_ago: format_updated_ago(&updated_at),
                updated_at,
            });
        } else if is_mcp_output_format(&proof.format) {
            let Some((source, pack_file)) = parse_framed_asset_id("mcp", &proof.asset_id) else {
                return Err(err(format!(
                    "installed MCP proof has malformed asset id: {}",
                    proof.asset_id
                )));
            };
            if listed_mcps.insert((proof.asset_id.clone(), proof.unit.clone())) {
                inventory.mcps.push(AssetRow {
                    name: proof.unit.clone(),
                    scope,
                    pack_file: pack_file.to_string(),
                    source: source.to_string(),
                });
            }
        } else {
            let Some((source, name)) = parse_framed_asset_id("command", &proof.asset_id) else {
                return Err(err(format!(
                    "installed command proof has malformed asset id: {}",
                    proof.asset_id
                )));
            };
            if listed_commands.insert(proof.asset_id.clone()) {
                inventory.commands.push(AssetRow {
                    name: name.to_string(),
                    scope,
                    pack_file: String::new(),
                    source: source.to_string(),
                });
            }
        }
    }

    inventory
        .skills
        .sort_by_cached_key(|skill| skill.name.to_lowercase());
    inventory
        .mcps
        .sort_by_cached_key(|asset| asset.name.to_lowercase());
    inventory
        .commands
        .sort_by_cached_key(|asset| asset.name.to_lowercase());
    inventory.install_paths.sort();
    inventory.install_paths.dedup();
    inventory.issues.sort();
    inventory.issues.dedup();
    Ok(inventory)
}

fn live_proof_issue(proof: &ManagedOutput, scope: Scope) -> Result<Option<String>> {
    let destination = PathBuf::from(&proof.destination);
    let current = if proof.format == "skill-tree" {
        match inspect_skill_directory_state(&destination, scope)? {
            OriginalState::Missing => None,
            OriginalState::Directory(hash, _) => Some(hash),
            OriginalState::File(_, _) => None,
        }
    } else if is_mcp_output_format(&proof.format) {
        match inspect_file_state(&destination)? {
            OriginalState::Missing => None,
            OriginalState::File(_, _) => {
                let target = McpSettingsTarget {
                    path: destination.clone(),
                    format: parse_mcp_output_format(&proof.format)?,
                };
                current_mcp_fragment_hashes(std::slice::from_ref(&proof.unit), &target)?
                    .remove(&proof.unit)
                    .flatten()
            }
            OriginalState::Directory(_, _) => unreachable!(),
        }
    } else {
        match inspect_file_state(&destination)? {
            OriginalState::Missing => None,
            OriginalState::File(hash, _) => Some(hash),
            OriginalState::Directory(_, _) => None,
        }
    };
    Ok(match current {
        Some(hash) if hash == proof.hash => None,
        Some(_) => Some(format!(
            "managed output drifted from its ownership proof: {}",
            destination.display()
        )),
        None => Some(format!(
            "managed output is missing or has the wrong type: {}",
            destination.display()
        )),
    })
}

/// The merged proof-derived list view. With no scope override, global and project inventories
/// are merged; otherwise only the selected scope is inspected.
#[allow(clippy::type_complexity)]
pub fn load_skills_mcps_commands(
    scope_override: Option<Scope>,
    project_root: &Path,
    load_lock: &dyn Fn(Scope, &Path) -> Result<AgentLockFile>,
    load_runtime: &dyn Fn(Scope, &Path) -> Result<crate::runtime::RuntimeState>,
) -> Result<(Vec<InstalledSkill>, Vec<AssetRow>, Vec<AssetRow>)> {
    let inspect = |scope, composite| {
        let lock = load_lock(scope, project_root)?;
        let runtime = load_runtime(scope, project_root)?;
        inspect_installed_inventory(&lock, &runtime, scope, project_root, composite)
    };
    if let Some(scope) = scope_override {
        let inventory = inspect(scope, false)?;
        return Ok((inventory.skills, inventory.mcps, inventory.commands));
    }
    let mut global = inspect(Scope::Global, true)?;
    let project = inspect(Scope::Project, true)?;
    global.skills.extend(project.skills);
    global.mcps.extend(project.mcps);
    global.commands.extend(project.commands);
    global
        .skills
        .sort_by_cached_key(|skill| (scope_ord(skill.scope), skill.name.to_lowercase()));
    global
        .mcps
        .sort_by_cached_key(|asset| (asset.name.to_lowercase(), scope_ord(asset.scope)));
    global
        .commands
        .sort_by_cached_key(|asset| (asset.name.to_lowercase(), scope_ord(asset.scope)));
    Ok((global.skills, global.mcps, global.commands))
}

fn scope_ord(scope: Scope) -> u8 {
    match scope {
        Scope::Global => 0,
        Scope::Project => 1,
    }
}

fn scope_label(scope: Scope) -> &'static str {
    match scope {
        Scope::Global => "global",
        Scope::Project => "project",
    }
}

fn skill_display_id(lock_scope: Scope, raw_id: &str, composite: bool) -> String {
    if composite {
        format!("{}::{raw_id}", scope_label(lock_scope))
    } else {
        raw_id.to_string()
    }
}
// ===================================================================================
// clean (kasetto commands/clean.rs)
// ===================================================================================

/// Counts of what `clean` removed (or would remove).
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct CleanCounts {
    pub skills_removed: usize,
    pub mcps_removed: usize,
    pub commands_removed: usize,
}

#[cfg(test)]
fn decode_selector_targets(
    lock: &AgentLockFile,
    asset_id: &str,
    scope: Scope,
) -> Result<Vec<String>> {
    let selector = lock.source_selectors.get(asset_id).ok_or_else(|| {
        err(format!(
            "refusing agent clean: `{asset_id}` has no destination-bound selector"
        ))
    })?;
    let framed = selector.strip_prefix("v2|base=").ok_or_else(|| {
        err(format!(
            "refusing agent clean: `{asset_id}` selector is not fully framed"
        ))
    })?;
    let (base, rest) = decode_selector_frame(framed, asset_id, "base")?;
    if base.is_empty() {
        return Err(err(format!(
            "refusing agent clean: `{asset_id}` has an empty selector base"
        )));
    }
    let rest = rest.strip_prefix("|scope=").ok_or_else(|| {
        err(format!(
            "refusing agent clean: `{asset_id}` has no scope binding"
        ))
    })?;
    let scope_and_targets = match scope {
        Scope::Project => "project|targets=",
        Scope::Global => "global|targets=",
    };
    let rest = rest.strip_prefix(scope_and_targets).ok_or_else(|| {
        err(format!(
            "refusing agent clean: `{asset_id}` has a mismatched scope/target binding"
        ))
    })?;
    let (count, mut encoded) = rest.split_once('|').ok_or_else(|| {
        err(format!(
            "refusing agent clean: `{asset_id}` has no framed target payload"
        ))
    })?;
    let count = count.parse::<usize>().map_err(|_| {
        err(format!(
            "refusing agent clean: `{asset_id}` has an invalid target count"
        ))
    })?;
    if count == 0 || count > 4096 {
        return Err(err(format!(
            "refusing agent clean: `{asset_id}` has an invalid target count"
        )));
    }
    let mut targets = Vec::new();
    for _ in 0..count {
        let (target, rest) = decode_selector_frame(encoded, asset_id, "target")?;
        if target.is_empty() {
            return Err(err(format!(
                "refusing agent clean: `{asset_id}` has an invalid target list"
            )));
        }
        targets.push(target.to_string());
        encoded = rest;
    }
    if !encoded.is_empty() {
        return Err(err(format!(
            "refusing agent clean: `{asset_id}` selector has trailing target data"
        )));
    }
    Ok(targets)
}

#[cfg(test)]
fn decode_selector_frame<'a>(
    encoded: &'a str,
    asset_id: &str,
    field: &str,
) -> Result<(&'a str, &'a str)> {
    let colon = encoded.find(':').ok_or_else(|| {
        err(format!(
            "refusing agent clean: `{asset_id}` has malformed {field} framing"
        ))
    })?;
    let len = encoded[..colon].parse::<usize>().map_err(|_| {
        err(format!(
            "refusing agent clean: `{asset_id}` has malformed {field} length"
        ))
    })?;
    let encoded = &encoded[colon + 1..];
    if encoded.len() < len || !encoded.is_char_boundary(len) {
        return Err(err(format!(
            "refusing agent clean: `{asset_id}` has truncated {field} framing"
        )));
    }
    Ok(encoded.split_at(len))
}

fn collect_clean_proofs(
    lock: &AgentLockFile,
    updated: &UpdatedAt,
    scope: Scope,
    root: &Path,
) -> Vec<(String, ManagedOutput)> {
    if scope == Scope::Project {
        lock.installed_outputs
            .iter()
            .map(|(key, proof)| {
                (
                    key.clone(),
                    ManagedOutput {
                        asset_id: proof.asset_id.clone(),
                        destination: resolve_dest(&proof.destination, root)
                            .to_string_lossy()
                            .into_owned(),
                        format: proof.format.clone(),
                        unit: proof.unit.clone(),
                        hash: proof.hash.clone(),
                    },
                )
            })
            .collect()
    } else {
        updated
            .managed_outputs
            .iter()
            .map(|(key, proof)| (key.clone(), proof.clone()))
            .collect()
    }
}

fn validate_clean_proofs(proofs: &[(String, ManagedOutput)], scope: Scope) -> Result<()> {
    if scope != Scope::Global {
        return Ok(());
    }
    for (key, proof) in proofs {
        validate_runtime_proof_key(key, proof)?;
        let allowed = if is_mcp_output_format(&proof.format) {
            let destination = Path::new(&proof.destination);
            let home = dirs_home()?;
            let config = dirs_agent_env_config()?;
            parse_framed_asset_id("mcp", &proof.asset_id).is_some()
                && all_mcp_settings_targets(&home, &config)
                    .iter()
                    .any(|target| {
                        target.path == destination
                            && mcp_format_binding(target.format) == proof.format
                    })
        } else {
            global_runtime_output_allowed(proof)?
        };
        if !allowed {
            return Err(err(format!(
                "refusing unsupported global clean target: {}",
                proof.destination
            )));
        }
    }
    Ok(())
}
/// Validate and enumerate the exact ownership units a clean would remove, including retained
/// tombstones. Desired-only pins are not installed inventory and are simply cleared. Preview and
/// apply share this exact-proof plan so their summaries cannot diverge.
pub fn clean_actions(
    lock: &AgentLockFile,
    updated: &UpdatedAt,
    scope: Scope,
    project_root: &Path,
    configured_skill_roots: &[PathBuf],
) -> Result<(CleanCounts, Vec<Action>)> {
    if lock.version != LOCK_VERSION {
        return Err(err(format!(
            "agent clean requires a migrated v{LOCK_VERSION} lock with ownership attestations"
        )));
    }
    let root = scope_root(scope, project_root)?;
    validate_portable_output_claims(lock, scope, &root, configured_skill_roots)?;
    if scope == Scope::Project {
        validate_historical_custom_skill_claims(lock, updated, &root, configured_skill_roots)?;
    }
    let proofs = collect_clean_proofs(lock, updated, scope, &root);
    validate_clean_proofs(&proofs, scope)?;

    let mut counts = CleanCounts {
        skills_removed: 0,
        mcps_removed: 0,
        commands_removed: 0,
    };
    let mut actions = Vec::new();
    for (_, proof) in proofs {
        let (kind, source, name) = if proof.format == "skill-tree" {
            let (source, name) = parse_framed_asset_id("skill", &proof.asset_id)
                .ok_or_else(|| err("malformed proven skill identity"))?;
            counts.skills_removed += 1;
            ("skill", source, name)
        } else if is_mcp_output_format(&proof.format) {
            let (source, name) = parse_framed_asset_id("mcp", &proof.asset_id)
                .ok_or_else(|| err("malformed proven MCP identity"))?;
            counts.mcps_removed += 1;
            ("mcp", source, name)
        } else {
            let (source, name) = parse_framed_asset_id("command", &proof.asset_id)
                .ok_or_else(|| err("malformed proven command identity"))?;
            counts.commands_removed += 1;
            ("command", source, name)
        };
        actions.push(Action {
            source: Some(source.to_string()),
            skill: Some(match kind {
                "skill" => name.to_string(),
                "mcp" => format!("mcp:{}", proof.unit),
                _ => format!("command:{name}"),
            }),
            status: String::new(),
            error: None,
        });
    }
    Ok((counts, actions))
}

/// Proof-check and atomically remove every managed output together with the portable lock and
/// machine runtime ownership ledger. Any missing proof or differing content fails before commit.
pub fn clean_apply_transaction(
    lock: &mut AgentLockFile,
    updated: &mut UpdatedAt,
    scope: Scope,
    project_root: &Path,
    configured_skill_roots: &[PathBuf],
) -> Result<()> {
    clean_apply_transaction_with_fault(
        lock,
        updated,
        scope,
        project_root,
        configured_skill_roots,
        None,
        None,
    )
}

fn clean_apply_transaction_with_fault(
    lock: &mut AgentLockFile,
    updated: &mut UpdatedAt,
    scope: Scope,
    project_root: &Path,
    configured_skill_roots: &[PathBuf],
    output_fault: Option<TransactionFault>,
    state_fault: Option<TransactionFault>,
) -> Result<()> {
    if lock.version != LOCK_VERSION {
        return Err(err(format!(
            "agent clean requires a migrated v{LOCK_VERSION} lock with ownership attestations"
        )));
    }
    let root = scope_root(scope, project_root)?;
    validate_portable_output_claims(lock, scope, &root, configured_skill_roots)?;
    if scope == Scope::Project {
        validate_historical_custom_skill_claims(lock, updated, &root, configured_skill_roots)?;
    }
    let proofs = collect_clean_proofs(lock, updated, scope, &root);
    validate_clean_proofs(&proofs, scope)?;

    let mut replacements = Vec::new();
    let mut mcp_groups = BTreeMap::<(PathBuf, String), Vec<ManagedOutput>>::new();
    for (key, proof) in proofs {
        if scope == Scope::Global {
            validate_runtime_proof_key(&key, &proof)?;
        }
        let destination = PathBuf::from(&proof.destination);
        if scope == Scope::Project {
            validate_managed_destination(&destination, scope, &root)?;
        }
        if is_mcp_output_format(&proof.format) {
            if scope == Scope::Global {
                let home = dirs_home()?;
                let config = dirs_agent_env_config()?;
                if !all_mcp_settings_targets(&home, &config)
                    .iter()
                    .any(|target| {
                        target.path == destination
                            && mcp_format_binding(target.format) == proof.format
                    })
                {
                    return Err(err(format!(
                        "refusing unsupported global MCP clean target: {}",
                        destination.display()
                    )));
                }
            }
            mcp_groups
                .entry((destination, proof.format.clone()))
                .or_default()
                .push(proof);
            continue;
        }
        if scope == Scope::Global && !global_runtime_output_allowed(&proof)? {
            return Err(err(format!(
                "refusing unsupported global clean target: {}",
                destination.display()
            )));
        }
        let original = if proof.format == "skill-tree" {
            inspect_skill_directory_state(&destination, scope)?
        } else {
            inspect_file_state(&destination)?
        };
        match &original {
            OriginalState::Missing => {}
            OriginalState::Directory(hash, _) | OriginalState::File(hash, _)
                if hash == &proof.hash =>
            {
                replacements.push(LockedReplacement {
                    destination,
                    original,
                    payload: ReplacementPayload::Delete,
                });
            }
            _ => {
                return Err(err(format!(
                    "refusing to clean drifted managed output at {}",
                    destination.display()
                )));
            }
        }
    }

    for ((destination, format), proofs) in mcp_groups {
        let target = McpSettingsTarget {
            path: destination.clone(),
            format: parse_mcp_output_format(&format)?,
        };
        let original = inspect_file_state(&destination)?;
        let names = proofs
            .iter()
            .map(|proof| proof.unit.clone())
            .collect::<Vec<_>>();
        let current = current_mcp_fragment_hashes(&names, &target)?;
        let mut present = Vec::new();
        for proof in proofs {
            if let Some(hash) = current.get(&proof.unit).and_then(Clone::clone) {
                if hash != proof.hash {
                    return Err(err(format!(
                        "refusing to clean drifted MCP `{}` at {}",
                        proof.unit,
                        destination.display()
                    )));
                }
                present.push(proof.unit);
            }
        }
        if !present.is_empty() {
            let current_bytes = fs::read(&destination)?;
            let rendered = render_mcp_settings_bytes(
                br#"{"mcpServers":{}}"#,
                Some(&current_bytes),
                target.format,
                &present,
            )?;
            replacements.push(LockedReplacement {
                destination,
                original,
                payload: ReplacementPayload::File {
                    bytes: rendered,
                    new_mode: if scope == Scope::Global { 0o600 } else { 0o644 },
                },
            });
        }
    }

    let mut cleared = lock.clone();
    cleared.clear_all();
    let lock_bytes = serde_yaml::to_string(&cleared)
        .map_err(|error| err(format!("failed to serialize cleared agent lock: {error}")))?
        .into_bytes();
    let lock_destination = lock_path(scope, project_root, &dirs_agent_env_data()?);
    let lock_original = inspect_file_state(&lock_destination)?;
    let output_count = replacements.len();
    replacements.push(LockedReplacement {
        destination: lock_destination,
        original: lock_original,
        payload: ReplacementPayload::File {
            bytes: lock_bytes,
            new_mode: if scope == Scope::Global { 0o600 } else { 0o644 },
        },
    });

    let runtime_destination = runtime_state_path(scope, project_root)?;
    let runtime_original = inspect_file_state(&runtime_destination)?;
    if !matches!(runtime_original, OriginalState::Missing) {
        replacements.push(LockedReplacement {
            destination: runtime_destination,
            original: runtime_original,
            payload: ReplacementPayload::Delete,
        });
    }
    let fault = output_fault
        .or_else(|| state_fault.map(|fault| offset_transaction_fault(fault, output_count)));
    // Outputs and both ownership roots share one strict commit/rollback boundary. In-memory
    // ownership is cleared only after every replacement and backup cleanup succeeds.
    commit_replacements_inner(&replacements, fault, true).map_err(|failure| failure.error)?;
    *lock = cleared;
    *updated = UpdatedAt::default();
    Ok(())
}

// ===================================================================================
// add / remove edit planning (kasetto commands/add.rs + remove.rs)
// ===================================================================================

/// One resolved section edit: which list, and the entry to insert there.
pub struct SectionEdit {
    pub section: Section,
    pub item: SourceItem,
}

/// Decompose the positional source + flags into the per-section edits `add` will apply.
/// Ported from kasetto `add.rs::{resolve_pin, plan_edits, selector_from}` + browse-URL derivation.
#[allow(clippy::too_many_arguments)]
pub fn plan_add_edits(
    raw_source: &str,
    at_ref: Option<&str>,
    skills: &[String],
    mcps: &[String],
    commands: &[String],
    git_ref: Option<&str>,
    branch: Option<&str>,
    sub_dir: Option<&str>,
) -> (String, Pin, Option<String>, Vec<SectionEdit>) {
    let derived = derive_browse_url(raw_source).unwrap_or_else(|| BrowseDerived {
        source: raw_source.to_string(),
        ..Default::default()
    });
    let source = derived.source.clone();
    let pin = resolve_pin(git_ref, branch, at_ref, &derived);
    let resolved_sub_dir = sub_dir
        .map(str::to_string)
        .or_else(|| derived.sub_dir.clone());

    let skill_names: Vec<String> = if !skills.is_empty() {
        skills.to_vec()
    } else if let Some(name) = &derived.skill_name {
        vec![name.clone()]
    } else {
        Vec::new()
    };
    let nothing_specified = skill_names.is_empty() && mcps.is_empty() && commands.is_empty();

    let mut edits = Vec::new();
    let mut push = |section: Section, selector: Selector| {
        let item_sub = if section == Section::Mcps {
            None
        } else {
            resolved_sub_dir.clone()
        };
        edits.push(SectionEdit {
            section,
            item: SourceItem {
                source: source.clone(),
                pin: pin.clone(),
                sub_dir: item_sub,
                selector,
            },
        });
    };

    if !skill_names.is_empty() {
        push(Section::Skills, selector_from(&skill_names));
    } else if nothing_specified {
        push(Section::Skills, Selector::Wildcard);
    }
    if !mcps.is_empty() {
        push(Section::Mcps, selector_from(mcps));
    }
    if !commands.is_empty() {
        push(Section::Commands, selector_from(commands));
    }
    (source, pin, resolved_sub_dir, edits)
}

fn resolve_pin(
    git_ref: Option<&str>,
    branch: Option<&str>,
    at_ref: Option<&str>,
    derived: &BrowseDerived,
) -> Pin {
    if let Some(r) = git_ref {
        return Pin::Ref(r.to_string());
    }
    if let Some(b) = branch {
        return Pin::Branch(b.to_string());
    }
    if let Some(r) = at_ref {
        return Pin::Ref(r.to_string());
    }
    if let Some(r) = &derived.git_ref {
        return Pin::Ref(r.clone());
    }
    if let Some(b) = &derived.branch {
        return Pin::Branch(b.clone());
    }
    Pin::None
}

fn selector_from(names: &[String]) -> Selector {
    if names.len() == 1 && names[0] == "*" {
        Selector::Wildcard
    } else {
        Selector::Names(names.to_vec())
    }
}

/// Fetch the source once to confirm it resolves before touching the config; for named
/// skill entries also assert each skill exists. Ported from kasetto `add.rs::verify_source`.
pub fn verify_source(
    source: &str,
    pin: &Pin,
    sub_dir: Option<&str>,
    edits: &[SectionEdit],
    config_path: &Path,
) -> Result<()> {
    let spec = SourceSpec {
        source: source.to_string(),
        branch: match pin {
            Pin::Branch(b) => Some(b.clone()),
            _ => None,
        },
        git_ref: match pin {
            Pin::Ref(r) => Some(r.clone()),
            _ => None,
        },
        sub_dir: sub_dir.map(str::to_string),
        skills: SkillsField::Wildcard("*".to_string()),
    };
    let cfg_dir = config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let stage = std::env::temp_dir().join(format!("envctl-agent-add-{}", now_unix()));

    let materialized = materialize_source(&spec, &cfg_dir, &stage)?;

    let mut name_error = None;
    if let Some(names) = named_skills(edits) {
        let sf = SkillsField::List(names.iter().cloned().map(SkillTarget::Name).collect());
        match select_targets(&sf, &materialized.available, &materialized.source_root) {
            Ok((_, broken)) => {
                if let Some(b) = broken.first() {
                    name_error = Some(err(format!("skill `{}` not found in {source}", b.name)));
                }
            }
            Err(e) => name_error = Some(e),
        }
    }

    if let Some(dir) = materialized.cleanup_dir {
        let _ = fs::remove_dir_all(dir);
    }
    match name_error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn named_skills(edits: &[SectionEdit]) -> Option<&Vec<String>> {
    edits
        .iter()
        .find_map(|e| match (&e.section, &e.item.selector) {
            (Section::Skills, Selector::Names(names)) => Some(names),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Agent, AgentField};
    use crate::hash::hash_file;
    use crate::lock::LOCK_FILENAME;

    fn temp_dir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("{name}-{}-{}", std::process::id(), now_unix()));
        let _ = fs::remove_dir_all(&p);
        p
    }

    fn bound_selector(targets: &[String]) -> String {
        bind_selector_targets("v1|test=bound", Scope::Project, targets)
    }

    fn insert_portable_proof(
        lock: &mut AgentLockFile,
        asset_id: &str,
        destination: &str,
        format: &str,
        unit: &str,
        hash: &str,
    ) {
        let key = installed_output_key(asset_id, destination, format, unit);
        lock.installed_outputs.insert(
            key,
            InstalledOutput {
                asset_id: asset_id.into(),
                destination: destination.into(),
                format: format.into(),
                unit: unit.into(),
                hash: hash.into(),
            },
        );
    }

    fn skill_entry_for(source: &str, name: &str, destination: &str, hash: &str) -> AgentLockEntry {
        AgentLockEntry {
            destination: destination.into(),
            hash: hash.into(),
            skill: name.into(),
            description: String::new(),
            source: source.into(),
            source_revision: "local".into(),
            scope: Some(Scope::Project),
        }
    }

    struct TestXdg {
        _guard: std::sync::MutexGuard<'static, ()>,
        home: Option<std::ffi::OsString>,
        cache: Option<std::ffi::OsString>,
        data: Option<std::ffi::OsString>,
    }

    impl TestXdg {
        fn pin(root: &Path) -> Self {
            let guard = crate::dirs::test_env_lock();
            let home = std::env::var_os("HOME");
            let cache = std::env::var_os("XDG_CACHE_HOME");
            let data = std::env::var_os("XDG_DATA_HOME");
            let test_home = root.join("test-home");
            let test_cache = root.join("test-cache");
            let test_data = root.join("test-data");
            fs::create_dir_all(&test_home).unwrap();
            fs::create_dir_all(&test_cache).unwrap();
            fs::create_dir_all(&test_data).unwrap();
            std::env::set_var("HOME", test_home);
            std::env::set_var("XDG_CACHE_HOME", test_cache);
            std::env::set_var("XDG_DATA_HOME", test_data);
            Self {
                _guard: guard,
                home,
                cache,
                data,
            }
        }
    }

    impl Drop for TestXdg {
        fn drop(&mut self) {
            for (name, value) in [
                ("HOME", self.home.take()),
                ("XDG_CACHE_HOME", self.cache.take()),
                ("XDG_DATA_HOME", self.data.take()),
            ] {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }

    #[test]
    fn bound_selector_is_unambiguous_with_delimiter_bearing_base_and_target_values() {
        let root = temp_dir("agent-env-framed-selector-delimiters");
        fs::create_dir_all(&root).unwrap();
        let cfg: Config = serde_yaml::from_str(
            r#"
scope: project
destination: "./custom|targets=5:other"
skills:
  - source: ./pack
    sub-dir: "sub|targets=5:other"
    skills:
      - name: alpha
        path: "path|targets=5:other"
"#,
        )
        .unwrap();
        let targets = vec![root.join("custom|targets=5:other")];
        let selector =
            skill_lock_selector(&cfg.skills[0], "alpha", Scope::Project, &targets, &root)
                .unwrap()
                .unwrap();
        assert!(selector.starts_with("v2|base="));
        assert!(selector.contains("sub|targets=5:other"));
        assert!(selector.contains("path|targets=5:other"));

        let id = skill_key("./pack", "alpha");
        let mut lock = AgentLockFile::default();
        lock.source_selectors.insert(id.clone(), selector);
        assert_eq!(
            decode_selector_targets(&lock, &id, Scope::Project).unwrap(),
            vec!["custom|targets=5:other/alpha"]
        );

        lock.source_selectors.insert(
            id.clone(),
            "v1|kind=skill|sub-dir=-|selection=name|scope=project|targets=36:custom|targets=5:other/alpha"
                .into(),
        );
        assert!(decode_selector_targets(&lock, &id, Scope::Project)
            .unwrap_err()
            .to_string()
            .contains("not fully framed"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn driver_ctx_from_mode_maps_lock_modes() {
        let cfg = Config {
            destination: None,
            scope: Some(Scope::Project),
            runtime: None,
            agent: None,
            skills: Vec::new(),
            mcps: Vec::new(),
            commands: Vec::new(),
        };
        let dests: Vec<PathBuf> = Vec::new();
        let root = PathBuf::from("/tmp");

        let plain = DriverCtx::from_mode(
            &cfg,
            &root,
            &dests,
            root.clone(),
            Scope::Project,
            false,
            &LockMode::Plain,
        );
        assert!(plain.dry_run, "apply=false => dry_run");
        assert!(!plain.locked);
        assert!(!plain.update);

        let locked = DriverCtx::from_mode(
            &cfg,
            &root,
            &dests,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Locked,
        );
        assert!(!locked.dry_run, "apply=true => not dry_run");
        assert!(locked.locked);

        let upd = DriverCtx::from_mode(
            &cfg,
            &root,
            &dests,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Update(vec!["a".into()]),
        );
        assert!(upd.update);
        assert_eq!(upd.update_only, vec!["a".to_string()]);
    }

    #[test]
    fn rebuild_lock_rejects_future_schema_before_relabelling() {
        let root = temp_dir("agent-env-future-lock-rebuild");
        fs::create_dir_all(&root).unwrap();
        let cfg = Config {
            destination: None,
            scope: Some(Scope::Project),
            runtime: None,
            agent: None,
            skills: Vec::new(),
            mcps: Vec::new(),
            commands: Vec::new(),
        };
        let previous = AgentLockFile {
            version: LOCK_VERSION + 1,
            ..AgentLockFile::default()
        };
        let before = previous.clone();

        let message = rebuild_lock(&cfg, &root, Scope::Project, &previous, &[])
            .unwrap_err()
            .to_string();
        assert!(message.contains("newer than supported"), "{message}");
        assert_eq!(previous, before);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn selective_update_materializes_and_verifies_every_remote_desired_source() {
        use std::cell::RefCell;

        let root = temp_dir("agent-env-selective-update-complete-inputs");
        let selected_root = root.join("selected-source");
        let unselected_root = root.join("unselected-source");
        for (source_root, name) in [
            (&selected_root, "selected"),
            (&unselected_root, "unselected"),
        ] {
            fs::create_dir_all(source_root.join(name)).unwrap();
            fs::write(
                source_root.join(name).join("SKILL.md"),
                format!("# {name}\n"),
            )
            .unwrap();
        }
        let cfg: Config = serde_yaml::from_str(
            r#"
scope: project
destination: ./dest
skills:
  - source: https://example.invalid/selected
    ref: pinned
    skills: [selected]
  - source: https://example.invalid/unselected
    ref: pinned
    skills: [unselected]
"#,
        )
        .unwrap();
        let destinations = vec![root.join("dest")];
        let ctx = DriverCtx {
            cfg: &cfg,
            cfg_dir: &root,
            destinations: &destinations,
            scope_root: root.clone(),
            scope: Scope::Project,
            dry_run: false,
            update: true,
            update_only: vec!["selected".into()],
            locked: false,
        };
        let mut desired = AgentLockFile::default();
        for (url, source_root, name) in [
            (
                "https://example.invalid/selected",
                &selected_root,
                "selected",
            ),
            (
                "https://example.invalid/unselected",
                &unselected_root,
                "unselected",
            ),
        ] {
            let hash = skill_snapshot_for_scope(
                TreeSnapshot::capture_destination(&source_root.join(name)).unwrap(),
                Scope::Project,
            )
            .unwrap()
            .hash();
            let mut entry = skill_entry_for(url, name, &format!("dest/{name}"), &hash);
            entry.source_revision = "ref:pinned".into();
            desired.skills.insert(skill_key(url, name), entry);
        }

        let calls = RefCell::new(Vec::new());
        let materialize = |source: &SourceSpec, _cfg_dir: &Path, _stage: &Path| {
            calls.borrow_mut().push(source.source.clone());
            let (source_root, name) = if source.source == "https://example.invalid/selected" {
                (&selected_root, "selected")
            } else {
                (&unselected_root, "unselected")
            };
            Ok(crate::source::MaterializedSource {
                source_revision: "ref:pinned".into(),
                available: HashMap::from([(name.to_string(), source_root.join(name))]),
                source_root: source_root.clone(),
                cleanup_dir: None,
            })
        };
        let mut inputs = LockedInputs::default();
        materialize_remote_inputs_with(&ctx, &desired, &mut inputs, &materialize).unwrap();

        assert_eq!(
            calls.into_inner(),
            vec![
                "https://example.invalid/selected".to_string(),
                "https://example.invalid/unselected".to_string(),
            ],
            "selective update may choose which lock pins move, but its atomic install plan must verify every remote desired input"
        );
        assert!(inputs
            .skills
            .contains_key(&skill_key("https://example.invalid/selected", "selected")));
        assert!(inputs.skills.contains_key(&skill_key(
            "https://example.invalid/unselected",
            "unselected"
        )));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn installed_inventory_requires_live_matching_ownership_proofs() {
        let root = temp_dir("agent-env-proof-derived-inventory");
        let destination = root.join(".claude/skills/alpha");
        fs::create_dir_all(&destination).unwrap();
        fs::write(
            destination.join("SKILL.md"),
            "---\nname: alpha\ndescription: proven\n---\n# Alpha\n",
        )
        .unwrap();
        let hash = skill_snapshot_for_scope(
            TreeSnapshot::capture_destination(&destination).unwrap(),
            Scope::Project,
        )
        .unwrap()
        .hash();
        let id = skill_key("./pack", "alpha");
        let mut desired_only = AgentLockFile::default();
        desired_only.skills.insert(
            id.clone(),
            skill_entry_for("./pack", "alpha", ".claude/skills/alpha", &hash),
        );

        let unproven = inspect_installed_inventory(
            &desired_only,
            &crate::runtime::RuntimeState::default(),
            Scope::Project,
            &root,
            false,
        )
        .unwrap();
        assert!(unproven.skills.is_empty());
        assert!(unproven
            .issues
            .iter()
            .any(|issue| issue.contains("no installed ownership proof")));

        let mut proven = desired_only.clone();
        insert_portable_proof(
            &mut proven,
            &id,
            ".claude/skills/alpha",
            "skill-tree",
            "tree",
            &hash,
        );
        let installed = inspect_installed_inventory(
            &proven,
            &crate::runtime::RuntimeState::default(),
            Scope::Project,
            &root,
            false,
        )
        .unwrap();
        assert!(installed.issues.is_empty(), "{:?}", installed.issues);
        assert_eq!(installed.skills.len(), 1);
        assert_eq!(installed.skills[0].skill, "alpha");
        assert_eq!(
            installed.install_paths,
            vec![root.join(".claude/skills").to_string_lossy().into_owned()]
        );

        fs::write(destination.join("SKILL.md"), "# drifted\n").unwrap();
        let drifted = inspect_installed_inventory(
            &proven,
            &crate::runtime::RuntimeState::default(),
            Scope::Project,
            &root,
            false,
        )
        .unwrap();
        assert!(drifted.skills.is_empty());
        assert!(drifted
            .issues
            .iter()
            .any(|issue| issue.contains("drifted from its ownership proof")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plan_add_edits_defaults_to_skills_wildcard() {
        let (source, _pin, _sub, edits) = plan_add_edits(
            "https://example.com/pack",
            None,
            &[],
            &[],
            &[],
            None,
            None,
            None,
        );
        assert_eq!(source, "https://example.com/pack");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].section, Section::Skills);
        assert!(matches!(edits[0].item.selector, Selector::Wildcard));
    }

    #[test]
    fn plan_add_edits_named_mcps_and_commands() {
        let (_s, _p, _d, edits) = plan_add_edits(
            "https://example.com/pack",
            None,
            &[],
            &["github".into()],
            &["review".into()],
            None,
            None,
            None,
        );
        // No skills specified but mcps+commands ARE → no skills wildcard.
        assert_eq!(edits.len(), 2);
        assert!(edits.iter().any(|e| e.section == Section::Mcps));
        assert!(edits.iter().any(|e| e.section == Section::Commands));
        // MCP edits never carry sub-dir.
        let mcp = edits.iter().find(|e| e.section == Section::Mcps).unwrap();
        assert!(mcp.item.sub_dir.is_none());
    }

    #[test]
    fn sync_mcps_replaces_locked_local_server_when_source_hash_changes() {
        let root = temp_dir("agent-env-driver-mcp-local-refresh");
        let _xdg = TestXdg::pin(&root);
        let source_dir = root.join("agent-skills/mcps");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(root.join(".codex")).unwrap();

        let source = source_dir.join("github.json");
        fs::write(
            &source,
            r#"{"mcpServers":{"github":{"command":"npx","args":["-y","old"]}}}"#,
        )
        .unwrap();
        let old_hash = hash_file(&source).unwrap();
        fs::write(
            &source,
            r#"{"mcpServers":{"github":{"command":"bash","args":["-lc","exec \"$META_ROOT/usr/bin/bunx\" @modelcontextprotocol/server-github"]}}}"#,
        )
        .unwrap();
        let new_hash = hash_file(&source).unwrap();
        assert_ne!(old_hash, new_hash);

        fs::write(
            root.join(".mcp.json"),
            r#"{"mcpServers":{"github":{"command":"npx","args":["-y","old"]},"weave":{"command":"weave"}}}"#,
        )
        .unwrap();
        fs::write(
            root.join(".codex/config.toml"),
            r#"[mcp_servers.github]
command = "npx"
args = ["-y", "old"]

[mcp_servers.weave]
command = "weave"
"#,
        )
        .unwrap();

        let cfg = Config {
            destination: None,
            scope: Some(Scope::Project),
            runtime: None,
            agent: Some(AgentField::Many(vec![Agent::ClaudeCode, Agent::Codex])),
            skills: Vec::new(),
            mcps: vec![crate::config::McpSourceSpec {
                source: "./agent-skills".into(),
                branch: None,
                git_ref: None,
                mcps: McpsField::List(vec![McpEntry::Name("github".into())]),
            }],
            commands: Vec::new(),
        };
        let mut lock = AgentLockFile::default();
        lock.save_tracked_asset(
            &mcp_asset_id("./agent-skills", "github.json"),
            AssetEntry {
                kind: "mcp".into(),
                name: "github.json".into(),
                hash: old_hash,
                source: "./agent-skills".into(),
                destination: "github".into(),
                source_revision: "local".into(),
            },
        );
        let destinations = Vec::new();
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut updated = UpdatedAt::default();
        for target in resolve_mcp_settings_targets(&cfg, Scope::Project, &root).unwrap() {
            let hashes = current_mcp_fragment_hashes(&["github".into()], &target).unwrap();
            record_runtime_ownership(
                &mut updated,
                &mcp_asset_id("./agent-skills", "github.json"),
                &target.path,
                mcp_format_binding(target.format),
                "github",
                hashes["github"].as_deref().unwrap(),
            );
        }
        let res = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(res.summary.failed, 0, "actions: {:?}", res.actions);
        assert_eq!(res.summary.updated, 1);

        let claude: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(claude["mcpServers"]["github"]["command"], "bash");
        assert_eq!(claude["mcpServers"]["weave"]["command"], "weave");

        let codex: toml::Value = fs::read_to_string(root.join(".codex/config.toml"))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(
            codex["mcp_servers"]["github"]["command"].as_str().unwrap(),
            "bash"
        );
        assert_eq!(
            codex["mcp_servers"]["weave"]["command"].as_str().unwrap(),
            "weave"
        );

        let (locked_hash, servers) = lock
            .get_tracked_asset("mcp", &mcp_asset_id("./agent-skills", "github.json"))
            .unwrap();
        assert_eq!(locked_hash, new_hash);
        assert_eq!(
            decode_asset_list(&servers, lock.version).unwrap(),
            ["github"]
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn zero_network_audit_never_self_certifies_unrecorded_remote_selectors() {
        let root = temp_dir("agent-env-driver-remote-selector-audit");
        fs::create_dir_all(&root).unwrap();
        let cfg: Config = serde_yaml::from_str(
            r#"scope: project
agent: claude-code
destination: ./out
skills:
  - source: https://example.invalid/org/repo
    ref: deadbeef
    sub-dir: changed-subdir
    skills:
      - name: alpha
        path: changed-path
"#,
        )
        .unwrap();
        let mut previous = AgentLockFile::default();
        let id = skill_key("https://example.invalid/org/repo", "alpha");
        previous.skills.insert(
            id.clone(),
            AgentLockEntry {
                destination: "out/alpha".into(),
                hash: "content-from-an-unknown-selector".into(),
                skill: "alpha".into(),
                description: "old selector".into(),
                source: "https://example.invalid/org/repo".into(),
                source_revision: "ref:deadbeef".into(),
                scope: Some(Scope::Project),
            },
        );

        let audited = audit_lock_zero_network(&cfg, &root, Scope::Project, &previous).unwrap();
        assert_eq!(
            previous.lock_check(&audited),
            vec![crate::lock::LockDrift {
                status: crate::lock::DriftStatus::Added,
                id: format!("selector::{id}"),
            }],
            "a legacy remote content hash cannot prove which sub-dir/path produced it"
        );

        previous.set_source_selector(
            &id,
            Some("v1|kind=skill|sub-dir=old|selection=object:old".into()),
        );
        let audited = audit_lock_zero_network(&cfg, &root, Scope::Project, &previous).unwrap();
        assert_eq!(
            previous.lock_check(&audited),
            vec![crate::lock::LockDrift {
                status: crate::lock::DriftStatus::Updated,
                id: format!("selector::{id}"),
            }],
            "changing a recorded remote selector must be typed drift"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zero_network_audit_records_remote_command_and_mcp_selectors() {
        let root = temp_dir("agent-env-driver-remote-asset-selector-audit");
        fs::create_dir_all(&root).unwrap();
        let cfg: Config = serde_yaml::from_str(
            r#"scope: project
agent: claude-code
commands:
  - source: https://example.invalid/org/commands
    ref: command-ref
    sub-dir: command-root
    commands:
      - name: review
        path: nested/review.md
mcps:
  - source: https://example.invalid/org/mcps
    ref: mcp-ref
    mcps:
      - name: servers
        path: nested/servers.json
"#,
        )
        .unwrap();
        let mut previous = AgentLockFile::default();
        let command_id = command_asset_id("https://example.invalid/org/commands", "review");
        previous.save_tracked_asset(
            &command_id,
            AssetEntry {
                kind: "command".into(),
                name: "review".into(),
                hash: "command-hash".into(),
                source: "https://example.invalid/org/commands".into(),
                destination: ".claude/commands/review.md".into(),
                source_revision: "ref:command-ref".into(),
            },
        );
        let mcp_id = mcp_asset_id("https://example.invalid/org/mcps", "servers.json");
        previous.save_tracked_asset(
            &mcp_id,
            AssetEntry {
                kind: "mcp".into(),
                name: "servers.json".into(),
                hash: "mcp-hash".into(),
                source: "https://example.invalid/org/mcps".into(),
                destination: "server".into(),
                source_revision: "ref:mcp-ref".into(),
            },
        );

        let audited = audit_lock_zero_network(&cfg, &root, Scope::Project, &previous).unwrap();
        let command_targets = resolve_command_targets(&cfg, Scope::Project, &root).unwrap();
        let mcp_targets = resolve_mcp_settings_targets(&cfg, Scope::Project, &root).unwrap();
        assert_eq!(
            audited.source_selectors.get(&command_id),
            command_lock_selector(
                &cfg.commands[0],
                "review",
                Scope::Project,
                &command_targets,
                &root
            )
            .unwrap()
            .as_ref()
        );
        assert_eq!(
            audited.source_selectors.get(&mcp_id),
            mcp_lock_selector(
                &cfg.mcps[0],
                "servers.json",
                Scope::Project,
                &mcp_targets,
                &root
            )
            .unwrap()
            .as_ref()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zero_network_audit_preserves_resolved_master_fallback_labels() {
        let root = temp_dir("agent-env-driver-master-fallback-audit");
        fs::create_dir_all(&root).unwrap();
        let cfg: Config = serde_yaml::from_str(
            r#"scope: project
agent: claude-code
destination: ./out
skills:
  - source: https://example.invalid/org/skills
    skills:
      - alpha
commands:
  - source: https://example.invalid/org/commands
    commands:
      - review
mcps:
  - source: https://example.invalid/org/mcps
    mcps:
      - servers
"#,
        )
        .unwrap();

        let mut previous = AgentLockFile::default();
        let skill_id = skill_key("https://example.invalid/org/skills", "alpha");
        previous.skills.insert(
            skill_id.clone(),
            AgentLockEntry {
                destination: "out/alpha".into(),
                hash: "skill-hash".into(),
                skill: "alpha".into(),
                description: "alpha".into(),
                source: "https://example.invalid/org/skills".into(),
                source_revision: "branch:master".into(),
                scope: Some(Scope::Project),
            },
        );
        let command_id = command_asset_id("https://example.invalid/org/commands", "review");
        previous.save_tracked_asset(
            &command_id,
            AssetEntry {
                kind: "command".into(),
                name: "review".into(),
                hash: "command-hash".into(),
                source: "https://example.invalid/org/commands".into(),
                destination: ".claude/commands/review.md".into(),
                source_revision: "branch:master".into(),
            },
        );
        let mcp_id = mcp_asset_id("https://example.invalid/org/mcps", "servers.json");
        previous.save_tracked_asset(
            &mcp_id,
            AssetEntry {
                kind: "mcp".into(),
                name: "servers.json".into(),
                hash: "mcp-hash".into(),
                source: "https://example.invalid/org/mcps".into(),
                destination: "server".into(),
                source_revision: "branch:master".into(),
            },
        );

        let audited = audit_lock_zero_network(&cfg, &root, Scope::Project, &previous).unwrap();
        assert_eq!(audited.skills[&skill_id].source_revision, "branch:master");
        assert_eq!(audited.assets[&command_id].source_revision, "branch:master");
        assert_eq!(audited.assets[&mcp_id].source_revision, "branch:master");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn skill_lock_rebuild_does_not_relabel_unmaterialized_remote_assets() {
        let root = temp_dir("agent-env-driver-no-asset-self-certification");
        fs::create_dir_all(&root).unwrap();
        let cfg: Config = serde_yaml::from_str(
            r#"scope: project
agent: claude-code
destination: ./out
commands:
  - source: https://example.invalid/org/commands
    ref: new-ref
    commands:
      - review
"#,
        )
        .unwrap();
        let id = command_asset_id("https://example.invalid/org/commands", "review");
        let mut previous = AgentLockFile::default();
        previous.save_tracked_asset(
            &id,
            AssetEntry {
                kind: "command".into(),
                name: "review".into(),
                hash: "bytes-from-old-ref".into(),
                source: "https://example.invalid/org/commands".into(),
                destination: "out/review".into(),
                source_revision: "ref:old-ref".into(),
            },
        );
        previous.set_source_selector(&id, Some("v1|kind=command|sub-dir=-|selection=name".into()));

        let selective = vec!["unrelated".into()];
        let rebuilt = rebuild_lock(&cfg, &root, Scope::Project, &previous, &selective).unwrap();
        assert_eq!(rebuilt.assets[&id].source_revision, "ref:old-ref");
        assert_eq!(rebuilt.assets[&id].hash, "bytes-from-old-ref");
        let error = audit_lock_zero_network(&cfg, &root, Scope::Project, &rebuilt).unwrap_err();
        assert!(
            error.to_string().contains("not pinned to `ref:new-ref`"),
            "locked audit must reject the stale asset revision: {error}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clean_mixed_proven_and_desired_only_removes_only_proven_output() {
        let root = temp_dir("agent-env-clean-partial-proof");
        let skill_root = root.join(".claude/skills");
        fs::create_dir_all(skill_root.join("alpha")).unwrap();
        fs::create_dir_all(skill_root.join("beta")).unwrap();
        fs::write(skill_root.join("alpha/SKILL.md"), "alpha").unwrap();
        fs::write(skill_root.join("beta/SKILL.md"), "beta").unwrap();

        let mut lock = AgentLockFile::default();
        for name in ["alpha", "beta"] {
            let id = skill_key("./pack", name);
            let destination = format!(".claude/skills/{name}");
            let hash = skill_snapshot_for_scope(
                TreeSnapshot::capture_destination(&root.join(&destination)).unwrap(),
                Scope::Project,
            )
            .unwrap()
            .hash();
            lock.skills.insert(
                id.clone(),
                skill_entry_for("./pack", name, &destination, &hash),
            );
            lock.source_selectors.insert(
                id.clone(),
                bound_selector(std::slice::from_ref(&destination)),
            );
            if name == "alpha" {
                insert_portable_proof(&mut lock, &id, &destination, "skill-tree", "tree", &hash);
            }
        }
        let mut updated = UpdatedAt::default();
        let (counts, actions) = clean_actions(
            &lock,
            &updated,
            Scope::Project,
            &root,
            std::slice::from_ref(&skill_root),
        )
        .unwrap();
        assert_eq!(counts.skills_removed, 1);
        assert_eq!(actions.len(), 1);
        clean_apply_transaction(
            &mut lock,
            &mut updated,
            Scope::Project,
            &root,
            std::slice::from_ref(&skill_root),
        )
        .unwrap();
        assert!(lock.skills.is_empty());
        assert!(lock.installed_outputs.is_empty());
        assert!(!skill_root.join("alpha").exists());
        assert!(skill_root.join("beta").is_dir());
        assert!(crate::lock::load(&root.join(LOCK_FILENAME))
            .unwrap()
            .skills
            .is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clean_desired_only_lock_has_zero_actions_and_clears_without_deleting_output() {
        let root = temp_dir("agent-env-clean-desired-only");
        let skill_root = root.join(".claude/skills");
        let destination = skill_root.join("alpha");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("SKILL.md"), "unowned").unwrap();
        let id = skill_key("./pack", "alpha");
        let mut lock = AgentLockFile::default();
        lock.skills.insert(
            id.clone(),
            skill_entry_for("./pack", "alpha", ".claude/skills/alpha", "desired"),
        );
        lock.source_selectors
            .insert(id, bound_selector(&[".claude/skills/alpha".into()]));
        let mut updated = UpdatedAt::default();
        let (counts, actions) = clean_actions(
            &lock,
            &updated,
            Scope::Project,
            &root,
            std::slice::from_ref(&skill_root),
        )
        .unwrap();
        assert_eq!(counts.skills_removed, 0);
        assert_eq!(counts.commands_removed, 0);
        assert_eq!(counts.mcps_removed, 0);
        assert!(actions.is_empty());
        clean_apply_transaction(
            &mut lock,
            &mut updated,
            Scope::Project,
            &root,
            std::slice::from_ref(&skill_root),
        )
        .unwrap();
        assert!(lock.skills.is_empty());
        assert_eq!(
            fs::read_to_string(destination.join("SKILL.md")).unwrap(),
            "unowned"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn forged_portable_tombstone_cannot_nominate_cargo_toml() {
        let root = temp_dir("agent-env-forged-tombstone");
        fs::create_dir_all(&root).unwrap();
        let cargo = root.join("Cargo.toml");
        fs::write(&cargo, "[package]\nname='foreign'\n").unwrap();
        let mut lock = AgentLockFile::default();
        let id = command_asset_id("./evil", "Cargo");
        insert_portable_proof(
            &mut lock,
            &id,
            "Cargo.toml",
            "markdown-frontmatter",
            "file",
            &hash_file(&cargo).unwrap(),
        );
        let message = clean_actions(&lock, &UpdatedAt::default(), Scope::Project, &root, &[])
            .unwrap_err()
            .to_string();
        assert!(message.contains("outside known native managed targets"));
        assert_eq!(
            fs::read_to_string(&cargo).unwrap(),
            "[package]\nname='foreign'\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn removed_custom_skill_root_tombstone_uses_matching_runtime_proof() {
        let root = temp_dir("agent-env-custom-root-tombstone");
        let _xdg = TestXdg::pin(&root);
        let destination = root.join("custom-skills/alpha");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("SKILL.md"), "alpha").unwrap();
        let snapshot = skill_snapshot_for_scope(
            TreeSnapshot::capture_destination(&destination).unwrap(),
            Scope::Project,
        )
        .unwrap();
        let hash = snapshot.hash();
        let id = skill_key("./pack", "alpha");

        let mut previous = AgentLockFile::default();
        previous.skills.insert(
            id.clone(),
            skill_entry_for("./pack", "alpha", "custom-skills/alpha", &hash),
        );
        previous
            .source_selectors
            .insert(id.clone(), bound_selector(&["custom-skills/alpha".into()]));
        insert_portable_proof(
            &mut previous,
            &id,
            "custom-skills/alpha",
            "skill-tree",
            "tree",
            &hash,
        );
        let mut updated = UpdatedAt::default();
        record_runtime_ownership(&mut updated, &id, &destination, "skill-tree", "tree", &hash);

        let cfg: Config = serde_yaml::from_str("scope: project\nagent: claude-code\n").unwrap();
        let mut lock = rebuild_lock(&cfg, &root, Scope::Project, &previous, &[]).unwrap();
        assert!(lock.skills.is_empty());
        assert_eq!(lock.installed_outputs.len(), 1);
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.failed, 0, "{:?}", result.actions);
        assert!(!destination.exists());
        assert!(lock.installed_outputs.is_empty());
        let _ = crate::runtime::clear_runtime_state(Scope::Project, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn retired_global_custom_skill_root_uses_secure_runtime_proof() {
        let root = temp_dir("agent-env-global-custom-root-tombstone");
        let _xdg = TestXdg::pin(&root);
        let destination = root.join("outside-home-custom/alpha");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("SKILL.md"), "alpha").unwrap();
        let hash = TreeSnapshot::capture_destination(&destination)
            .unwrap()
            .hash();
        let id = skill_key("./retired-pack", "alpha");
        let mut updated = UpdatedAt::default();
        record_runtime_ownership(&mut updated, &id, &destination, "skill-tree", "tree", &hash);
        let cfg: Config = serde_yaml::from_str("scope: global\nagent: claude-code\n").unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Global).unwrap();
        let scope_root = scope_root(Scope::Global, &root).unwrap();
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            scope_root,
            Scope::Global,
            true,
            &LockMode::Plain,
        );
        let mut lock = AgentLockFile::default();
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.failed, 0, "{:?}", result.actions);
        assert!(!destination.exists());
        assert!(updated.managed_outputs.is_empty());
        let _ = crate::runtime::clear_runtime_state(Scope::Global, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn global_mcp_merge_preserves_existing_private_config_mode() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("agent-env-global-mcp-private-mode");
        let _xdg = TestXdg::pin(&root);
        let source = root.join("pack/mcps");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("servers.json"),
            r#"{"mcpServers":{"github":{"command":"github-mcp"}}}"#,
        )
        .unwrap();
        let cfg: Config = serde_yaml::from_str(&format!(
            "scope: global\nagent: codex\nmcps:\n  - source: {}\n    mcps: [servers]\n",
            root.join("pack").display()
        ))
        .unwrap();
        let home = dirs_home().unwrap();
        let target = home.join(".codex/config.toml");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "[mcp_servers.weave]\ncommand = \"weave\"\n").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Global).unwrap();
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            scope_root(Scope::Global, &root).unwrap(),
            Scope::Global,
            true,
            &LockMode::Plain,
        );
        let mut lock = AgentLockFile::default();
        let mut updated = UpdatedAt::default();
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.failed, 0, "{:?}", result.actions);
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let value: toml::Value = fs::read_to_string(&target).unwrap().parse().unwrap();
        assert_eq!(
            value["mcp_servers"]["weave"]["command"].as_str(),
            Some("weave")
        );
        assert_eq!(
            value["mcp_servers"]["github"]["command"].as_str(),
            Some("github-mcp")
        );
        let _ = crate::runtime::clear_runtime_state(Scope::Global, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn strict_late_backup_cleanup_failure_restores_every_replacement() {
        let root = temp_dir("agent-env-late-cleanup-rollback");
        fs::create_dir_all(&root).unwrap();
        let mut replacements = Vec::new();
        for (name, old, new) in [("a", "old-a", "new-a"), ("b", "old-b", "new-b")] {
            let destination = root.join(name);
            fs::write(&destination, old).unwrap();
            replacements.push(LockedReplacement {
                original: inspect_file_state(&destination).unwrap(),
                destination,
                payload: ReplacementPayload::File {
                    bytes: new.as_bytes().to_vec(),
                    new_mode: 0o644,
                },
            });
        }
        let nested = root.join("new/deep/output");
        replacements.push(LockedReplacement {
            destination: nested.clone(),
            original: OriginalState::Missing,
            payload: ReplacementPayload::File {
                bytes: b"new-output".to_vec(),
                new_mode: 0o644,
            },
        });
        let failure = commit_replacements_inner(
            &replacements,
            Some(TransactionFault::BackupCleanup(1)),
            true,
        )
        .unwrap_err();
        assert!(failure.rollback_complete, "{}", failure.error);
        assert_eq!(fs::read_to_string(root.join("a")).unwrap(), "old-a");
        assert_eq!(fs::read_to_string(root.join("b")).unwrap(), "old-b");
        assert!(!nested.exists());
        assert!(
            !root.join("new").exists(),
            "created parent namespace leaked"
        );
        assert!(fs::read_dir(&root).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("envctl-tx")
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn precommit_original_capture_failure_cleans_stages_and_created_parents() {
        let root = temp_dir("agent-env-precommit-capture-cleanup");
        fs::create_dir_all(&root).unwrap();
        let nested = root.join("new/deep/output");
        let absent_but_claimed_file = root.join("claimed-existing");
        let replacements = vec![
            LockedReplacement {
                destination: nested.clone(),
                original: OriginalState::Missing,
                payload: ReplacementPayload::File {
                    bytes: b"staged-output".to_vec(),
                    new_mode: 0o644,
                },
            },
            LockedReplacement {
                destination: absent_but_claimed_file,
                original: OriginalState::File("claimed-hash".into(), 0o644),
                payload: ReplacementPayload::File {
                    bytes: b"never-staged".to_vec(),
                    new_mode: 0o644,
                },
            },
        ];

        let failure = commit_replacements_inner(&replacements, None, true).unwrap_err();
        assert!(failure.rollback_complete, "{}", failure.error);
        assert!(!nested.exists());
        assert!(
            !root.join("new").exists(),
            "pre-commit failure leaked a created parent namespace"
        );
        assert!(fs::read_dir(&root).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("envctl-tx")
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn current_file_stage_write_failure_cleans_stage_and_created_parents() {
        let root = temp_dir("agent-env-current-file-stage-cleanup");
        fs::create_dir(&root).unwrap();
        let destination = root.join("new/deep/output");
        let replacements = vec![LockedReplacement {
            destination: destination.clone(),
            original: OriginalState::Missing,
            payload: ReplacementPayload::File {
                bytes: b"staged-output".to_vec(),
                new_mode: 0o644,
            },
        }];

        let failure = commit_replacements_inner(
            &replacements,
            Some(TransactionFault::FileStageWrite(0)),
            true,
        )
        .unwrap_err();
        assert!(failure.rollback_complete, "{}", failure.error);
        assert!(!destination.exists());
        assert!(!root.join("new").exists());
        assert!(fs::read_dir(&root).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("envctl-tx")
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn current_tree_stage_write_and_capture_failures_clean_every_owned_artifact() {
        for fault in [
            TransactionFault::TreeStageWrite(0),
            TransactionFault::TreeStageCapture(0),
        ] {
            let root = temp_dir("agent-env-current-tree-stage-cleanup");
            fs::create_dir(&root).unwrap();
            let source = root.join("source");
            fs::create_dir(&source).unwrap();
            fs::write(source.join("SKILL.md"), "tree payload").unwrap();
            let snapshot = TreeSnapshot::capture_destination(&source).unwrap();
            let destination = root.join("new/deep/skill");
            let replacements = vec![LockedReplacement {
                destination: destination.clone(),
                original: OriginalState::Missing,
                payload: ReplacementPayload::Directory(snapshot),
            }];

            let failure = commit_replacements_inner(&replacements, Some(fault), true).unwrap_err();
            assert!(failure.rollback_complete, "{}", failure.error);
            assert!(!destination.exists());
            assert!(!root.join("new").exists());
            assert!(fs::read_dir(&root).unwrap().all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains("envctl-tx")
            }));
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn stage_collision_errors_preserve_the_foreign_object() {
        let root = temp_dir("agent-env-stage-collision-preserved");
        fs::create_dir(&root).unwrap();
        let file_stage = root.join("file-stage");
        fs::write(&file_stage, b"foreign-file").unwrap();
        assert!(create_new_staged_file(&file_stage, b"managed", 0o644).is_err());
        assert_eq!(fs::read(&file_stage).unwrap(), b"foreign-file");

        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("SKILL.md"), "managed tree").unwrap();
        let snapshot = TreeSnapshot::capture_destination(&source).unwrap();
        let tree_stage = root.join("tree-stage");
        fs::create_dir(&tree_stage).unwrap();
        fs::write(tree_stage.join("foreign-marker"), "preserve").unwrap();
        assert!(snapshot.materialize_staged(&tree_stage).is_err());
        assert_eq!(
            fs::read_to_string(tree_stage.join("foreign-marker")).unwrap(),
            "preserve"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rollback_reconstruction_failure_keeps_committed_output_durable() {
        let root = temp_dir("agent-env-rollback-reconstruction-recovery");
        fs::create_dir_all(&root).unwrap();
        let replacements = [("a", "old-a", "new-a"), ("b", "old-b", "new-b")]
            .into_iter()
            .map(|(name, old, new)| {
                let destination = root.join(name);
                fs::write(&destination, old).unwrap();
                LockedReplacement {
                    original: inspect_file_state(&destination).unwrap(),
                    destination,
                    payload: ReplacementPayload::File {
                        bytes: new.as_bytes().to_vec(),
                        new_mode: 0o644,
                    },
                }
            })
            .collect::<Vec<_>>();
        let failure = commit_replacements_inner(
            &replacements,
            Some(TransactionFault::BackupCleanupThenRollbackReconstruction {
                cleanup_index: 1,
                reconstruction_index: 0,
            }),
            true,
        )
        .unwrap_err();
        assert!(!failure.rollback_complete);
        assert_eq!(
            fs::read_to_string(root.join("a")).unwrap(),
            "new-a",
            "failed old-state reconstruction must not remove the committed new output"
        );
        assert_eq!(fs::read_to_string(root.join("b")).unwrap(), "old-b");
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn transaction_preserves_private_existing_mode_and_sets_new_state_mode() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("agent-env-transaction-file-modes");
        fs::create_dir_all(&root).unwrap();
        let existing = root.join("config.toml");
        fs::write(&existing, "old").unwrap();
        fs::set_permissions(&existing, fs::Permissions::from_mode(0o600)).unwrap();
        let new_state = root.join("runtime.json");
        let permissive_state = root.join("permissive-runtime.json");
        fs::write(&permissive_state, "old").unwrap();
        fs::set_permissions(&permissive_state, fs::Permissions::from_mode(0o644)).unwrap();
        let replacements = vec![
            LockedReplacement {
                original: inspect_file_state(&existing).unwrap(),
                destination: existing.clone(),
                payload: ReplacementPayload::File {
                    bytes: b"new".to_vec(),
                    new_mode: 0o644,
                },
            },
            LockedReplacement {
                original: OriginalState::Missing,
                destination: new_state.clone(),
                payload: ReplacementPayload::File {
                    bytes: b"{}".to_vec(),
                    new_mode: 0o600,
                },
            },
            LockedReplacement {
                original: inspect_file_state(&permissive_state).unwrap(),
                destination: permissive_state.clone(),
                payload: ReplacementPayload::File {
                    bytes: b"new".to_vec(),
                    new_mode: 0o600,
                },
            },
        ];
        commit_replacements_inner(&replacements, None, true).unwrap();
        assert_eq!(
            fs::metadata(existing).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(new_state).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(permissive_state).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn current_item_restore_failure_retains_recovery_backup() {
        let root = temp_dir("agent-env-current-restore-failure");
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("state");
        fs::write(&destination, "old").unwrap();
        let replacement = LockedReplacement {
            original: inspect_file_state(&destination).unwrap(),
            destination: destination.clone(),
            payload: ReplacementPayload::File {
                bytes: b"new".to_vec(),
                new_mode: 0o600,
            },
        };
        let failure = commit_replacements_inner(
            &[replacement],
            Some(TransactionFault::StageRenameAndRestore(0)),
            true,
        )
        .unwrap_err();
        assert!(!failure.rollback_complete);
        let backup = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.to_string_lossy().contains("envctl-tx-backup"))
            .expect("recovery backup");
        assert_eq!(fs::read_to_string(backup).unwrap(), "old");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clean_cleanup_failure_preserves_lock_proof_and_restores_skill() {
        let root = temp_dir("agent-env-clean-cleanup-failure");
        let skill_root = root.join(".claude/skills");
        let destination = skill_root.join("alpha");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("SKILL.md"), "alpha").unwrap();
        let hash = skill_snapshot_for_scope(
            TreeSnapshot::capture_destination(&destination).unwrap(),
            Scope::Project,
        )
        .unwrap()
        .hash();
        let id = skill_key("./pack", "alpha");
        let mut lock = AgentLockFile::default();
        lock.skills.insert(
            id.clone(),
            skill_entry_for("./pack", "alpha", ".claude/skills/alpha", &hash),
        );
        lock.source_selectors
            .insert(id.clone(), bound_selector(&[".claude/skills/alpha".into()]));
        insert_portable_proof(
            &mut lock,
            &id,
            ".claude/skills/alpha",
            "skill-tree",
            "tree",
            &hash,
        );
        let before = lock.clone();
        let mut updated = UpdatedAt::default();
        let failure = clean_apply_transaction_with_fault(
            &mut lock,
            &mut updated,
            Scope::Project,
            &root,
            std::slice::from_ref(&skill_root),
            Some(TransactionFault::BackupCleanup(0)),
            None,
        )
        .unwrap_err();
        assert!(failure.to_string().contains("backup cleanup failure"));
        assert_eq!(lock, before);
        assert!(destination.join("SKILL.md").is_file());
        assert!(!root.join(LOCK_FILENAME).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clean_state_cleanup_failure_restores_lock_and_runtime_proofs() {
        let root = temp_dir("agent-env-clean-state-cleanup-failure");
        let _xdg = TestXdg::pin(&root);
        let skill_root = root.join(".claude/skills");
        let destination = skill_root.join("alpha");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("SKILL.md"), "alpha").unwrap();
        let id = skill_key("./pack", "alpha");
        let hash = skill_snapshot_for_scope(
            TreeSnapshot::capture_destination(&destination).unwrap(),
            Scope::Project,
        )
        .unwrap()
        .hash();
        let mut lock = AgentLockFile::default();
        lock.skills.insert(
            id.clone(),
            skill_entry_for("./pack", "alpha", ".claude/skills/alpha", &hash),
        );
        lock.source_selectors
            .insert(id.clone(), bound_selector(&[".claude/skills/alpha".into()]));
        insert_portable_proof(
            &mut lock,
            &id,
            ".claude/skills/alpha",
            "skill-tree",
            "tree",
            &hash,
        );
        crate::lock::save(&mut lock, &root.join(LOCK_FILENAME)).unwrap();

        let mut updated = UpdatedAt::default();
        record_runtime_ownership(&mut updated, &id, &destination, "skill-tree", "tree", &hash);
        let runtime = crate::runtime::RuntimeState {
            last_run: Some("before-clean".into()),
            latest_report: None,
            installed_at: BTreeMap::new(),
            managed_outputs: updated.managed_outputs.clone(),
        };
        crate::runtime::save_runtime_state(&runtime, Scope::Project, &root).unwrap();

        let before_lock = lock.clone();
        let before_outputs = updated.managed_outputs.clone();
        let failure = clean_apply_transaction_with_fault(
            &mut lock,
            &mut updated,
            Scope::Project,
            &root,
            std::slice::from_ref(&skill_root),
            None,
            Some(TransactionFault::BackupCleanup(1)),
        )
        .unwrap_err();
        assert!(failure.to_string().contains("backup cleanup failure"));
        assert_eq!(lock, before_lock);
        assert_eq!(updated.managed_outputs, before_outputs);
        assert_eq!(
            fs::read_to_string(destination.join("SKILL.md")).unwrap(),
            "alpha"
        );
        assert_eq!(
            crate::lock::load(&root.join(LOCK_FILENAME)).unwrap(),
            before_lock
        );
        assert_eq!(
            crate::runtime::load_runtime_state(Scope::Project, &root)
                .unwrap()
                .managed_outputs,
            before_outputs
        );
        assert!(fs::read_dir(&root).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("envctl-tx")
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn lock_then_first_sync_reports_install_and_persists_top_level_result() {
        let root = temp_dir("agent-env-first-install-report");
        let _xdg = TestXdg::pin(&root);
        let source = root.join("pack/alpha");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "first").unwrap();
        let cfg: Config = serde_yaml::from_str(
            "scope: project\ndestination: ./installed\nskills:\n  - source: ./pack\n    skills: [alpha]\n",
        )
        .unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        let mut lock =
            rebuild_lock(&cfg, &root, Scope::Project, &AgentLockFile::default(), &[]).unwrap();
        assert!(
            lock.installed_outputs.is_empty(),
            "lock must not fabricate ownership"
        );
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut updated = UpdatedAt::default();
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.installed, 1, "{:?}", result.actions);
        assert_eq!(result.summary.updated, 0);
        assert_eq!(result.actions[0].status, "installed");
        let report: serde_json::Value =
            serde_json::from_str(updated.latest_report.as_deref().unwrap()).unwrap();
        assert_eq!(report["summary"]["installed"], 1);
        assert!(
            report.get("report").is_none(),
            "latest_report is the SyncResult itself"
        );
        assert!(!lock.installed_outputs.is_empty());

        let before_lock = lock.clone();
        let before_outputs = updated.managed_outputs.clone();
        let before_bytes = fs::read(root.join("installed/alpha/SKILL.md")).unwrap();
        fs::write(source.join("SKILL.md"), "second").unwrap();
        let plan = prepare_nonlocked_sync(&ctx, &lock, &updated).unwrap();
        let failed = apply_sync_plan_with_fault(
            &ctx,
            &mut lock,
            &mut updated,
            plan,
            Some(TransactionFault::BeforeCommit(0)),
        );
        assert_eq!(failed.summary.failed, 1);
        assert_eq!(
            fs::read(root.join("installed/alpha/SKILL.md")).unwrap(),
            before_bytes
        );
        assert_eq!(lock, before_lock);
        assert_eq!(updated.managed_outputs, before_outputs);
        let runtime = crate::runtime::load_runtime_state(Scope::Project, &root).unwrap();
        assert_eq!(runtime.managed_outputs, before_outputs);
        assert_eq!(runtime.load_latest_failures().len(), 1);

        let locked_ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Locked,
        );
        let locked_failure = sync(&locked_ctx, &mut lock, &mut updated);
        assert_eq!(locked_failure.actions[0].status, "locked_error");
        assert_eq!(
            crate::runtime::load_runtime_state(Scope::Project, &root)
                .unwrap()
                .load_latest_failures()
                .len(),
            1
        );
        let _ = crate::runtime::clear_runtime_state(Scope::Project, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plain_sync_reattests_a_stale_project_skill_proof_when_the_tree_is_locked_correct() {
        let root = temp_dir("agent-env-reattest-project-skill-proof");
        let _xdg = TestXdg::pin(&root);
        let source = root.join("pack/alpha");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "verified").unwrap();
        let cfg: Config = serde_yaml::from_str(
            "scope: project\ndestination: ./installed\nskills:\n  - source: ./pack\n    skills: [alpha]\n",
        )
        .unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut lock =
            rebuild_lock(&cfg, &root, Scope::Project, &AgentLockFile::default(), &[]).unwrap();
        let mut updated = UpdatedAt::default();
        assert_eq!(sync(&ctx, &mut lock, &mut updated).summary.installed, 1);

        let asset_id = skill_key("./pack", "alpha");
        let desired_hash = lock.skills[&asset_id].hash.clone();
        let proof = lock
            .installed_outputs
            .values_mut()
            .find(|proof| proof.asset_id == asset_id)
            .unwrap();
        proof.hash = "interrupted-before-proof-persist".into();

        let locked_ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Locked,
        );
        let locked = sync(&locked_ctx, &mut lock, &mut updated);
        assert_eq!(locked.actions[0].status, "locked_error");

        let repaired = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(repaired.summary.updated, 1, "{:?}", repaired.actions);
        assert_eq!(repaired.actions[0].status, "updated");
        assert!(lock
            .installed_outputs
            .values()
            .filter(|proof| proof.asset_id == asset_id)
            .all(|proof| proof.hash == desired_hash));
        let _ = crate::runtime::clear_runtime_state(Scope::Project, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn v2_evidence_matches_entry_fields_all_skill_targets_and_not_colliding_keys() {
        let root = temp_dir("agent-env-v2-entry-evidence");
        fs::create_dir_all(&root).unwrap();
        let cfg = Config {
            destination: None,
            scope: Some(Scope::Project),
            runtime: None,
            agent: Some(AgentField::Many(vec![Agent::ClaudeCode, Agent::Codex])),
            skills: Vec::new(),
            mcps: Vec::new(),
            commands: Vec::new(),
        };
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut v2 = AgentLockFile {
            version: 2,
            ..AgentLockFile::default()
        };
        let source = "source::with-delimiter";
        let skill_name = "alpha";
        let skill_id = skill_key(source, skill_name);
        v2.skills.insert(
            "legacy-key-does-not-matter".into(),
            skill_entry_for(source, skill_name, ".claude/skills/alpha", "desired"),
        );
        // This old delimiter-concatenated key looks plausible but names a different source.
        v2.skills.insert(
            format!("{source}::{skill_name}"),
            skill_entry_for(
                "source",
                "with-delimiter::alpha",
                ".claude/skills/with-delimiter::alpha",
                "desired",
            ),
        );
        for destination in &destinations {
            assert!(legacy_v2_exact_output_is_named(
                &v2,
                &skill_id,
                &destination.join(skill_name),
                &ctx,
                "tree",
                "desired",
                "desired",
            )
            .unwrap());
        }

        let command_id = command_asset_id(source, "review");
        v2.assets.insert(
            "old-command-key".into(),
            AssetEntry {
                kind: "command".into(),
                name: "review".into(),
                hash: "desired".into(),
                source: source.into(),
                destination: ".claude/commands/review.md".into(),
                source_revision: "local".into(),
            },
        );
        assert!(legacy_v2_exact_output_is_named(
            &v2,
            &command_id,
            &root.join(".claude/commands/review.md"),
            &ctx,
            "file",
            "desired",
            "desired",
        )
        .unwrap());

        let mcp_id = mcp_asset_id(source, "servers.json");
        v2.assets.insert(
            "old-mcp-key".into(),
            AssetEntry {
                kind: "mcp".into(),
                name: "servers.json".into(),
                hash: "desired".into(),
                source: source.into(),
                destination: "github".into(),
                source_revision: "local".into(),
            },
        );
        assert!(legacy_v2_exact_output_is_named(
            &v2,
            &mcp_id,
            &root.join(".mcp.json"),
            &ctx,
            "github",
            "desired",
            "desired",
        )
        .unwrap());
        assert!(!legacy_v2_exact_output_is_named(
            &v2,
            &mcp_id,
            &root.join(".mcp.json"),
            &ctx,
            "other",
            "desired",
            "desired",
        )
        .unwrap());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn v2_desired_only_missing_skill_reports_install_not_update() {
        let root = temp_dir("agent-env-v2-missing-skill-summary");
        let _xdg = TestXdg::pin(&root);
        let source = root.join("pack/alpha");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "alpha").unwrap();
        let cfg: Config = serde_yaml::from_str(
            "scope: project\ndestination: ./installed\nskills:\n  - source: ./pack\n    skills: [alpha]\n",
        )
        .unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        let mut lock = AgentLockFile {
            version: 2,
            ..AgentLockFile::default()
        };
        lock.skills.insert(
            "./pack::alpha".into(),
            AgentLockEntry {
                destination: "installed/alpha".into(),
                hash: crate::hash::hash_dir(&source).unwrap(),
                skill: "alpha".into(),
                description: String::new(),
                source: "./pack".into(),
                source_revision: "local".into(),
                scope: Some(Scope::Project),
            },
        );
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut updated = UpdatedAt::default();
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.installed, 1, "{:?}", result.actions);
        assert_eq!(result.summary.updated, 0);
        assert_eq!(result.actions[0].status, "installed");
        assert!(root.join("installed/alpha/SKILL.md").is_file());
        let _ = crate::runtime::clear_runtime_state(Scope::Project, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn v2_partial_two_target_mcp_migration_reports_update() {
        let root = temp_dir("agent-env-v2-partial-mcp-summary");
        let _xdg = TestXdg::pin(&root);
        let source_path = root.join("pack/mcps/servers.json");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        let source_bytes = br#"{"mcpServers":{"exa":{"command":"exa"}}}"#;
        fs::write(&source_path, source_bytes).unwrap();
        let cfg: Config = serde_yaml::from_str(
            "scope: project\nagent: [claude-code, codex]\nmcps:\n  - source: ./pack\n    mcps: [servers]\n",
        )
        .unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        let targets = resolve_mcp_settings_targets(&cfg, Scope::Project, &root).unwrap();
        assert_eq!(targets.len(), 2);
        let first = &targets[0];
        fs::create_dir_all(first.path.parent().unwrap()).unwrap();
        let rendered = render_mcp_settings_bytes(source_bytes, None, first.format, &[]).unwrap();
        fs::write(&first.path, rendered).unwrap();

        let desired =
            rebuild_lock(&cfg, &root, Scope::Project, &AgentLockFile::default(), &[]).unwrap();
        let (_, mut legacy_entry) = desired.assets.into_iter().next().unwrap();
        legacy_entry.destination = "exa".into();
        let mut lock = AgentLockFile {
            version: 2,
            ..AgentLockFile::default()
        };
        lock.assets.insert("legacy-mcp".into(), legacy_entry);
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut updated = UpdatedAt::default();
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.installed, 0, "{:?}", result.actions);
        assert_eq!(result.summary.updated, 1);
        assert_eq!(result.actions[0].status, "updated");
        assert!(targets.iter().all(|target| target.path.is_file()));
        let _ = crate::runtime::clear_runtime_state(Scope::Project, &root);
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn plain_v2_two_agent_skill_migration_normalizes_modes_and_writes_proofs() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("agent-env-v2-two-agent-migration");
        let _xdg = TestXdg::pin(&root);
        let source = root.join("pack/alpha");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "alpha").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o775)).unwrap();
        fs::set_permissions(source.join("SKILL.md"), fs::Permissions::from_mode(0o664)).unwrap();
        let hand_authored_harness = root.join(".agents/skills/hand-authored-harness");
        fs::create_dir_all(&hand_authored_harness).unwrap();
        fs::write(
            hand_authored_harness.join("SKILL.md"),
            "hand-authored; agent-env must preserve this tree",
        )
        .unwrap();
        let cfg: Config = serde_yaml::from_str(
            "scope: project\nagent: [claude-code, codex]\nskills:\n  - source: ./pack\n    skills: [alpha]\n",
        )
        .unwrap();
        let destinations = resolve_destinations(&root, &cfg, Scope::Project).unwrap();
        for skill_root in destinations.iter().take(1) {
            let destination = skill_root.join("alpha");
            fs::create_dir_all(&destination).unwrap();
            fs::write(destination.join("SKILL.md"), "alpha").unwrap();
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o775)).unwrap();
            fs::set_permissions(
                destination.join("SKILL.md"),
                fs::Permissions::from_mode(0o664),
            )
            .unwrap();
        }
        let mut lock = AgentLockFile {
            version: 2,
            ..AgentLockFile::default()
        };
        lock.skills.insert(
            "./pack::alpha".into(),
            AgentLockEntry {
                destination: ".claude/skills/alpha".into(),
                hash: crate::hash::hash_dir(&source).unwrap(),
                skill: "alpha".into(),
                description: String::new(),
                source: "./pack".into(),
                source_revision: "local".into(),
                scope: Some(Scope::Project),
            },
        );
        let ctx = DriverCtx::from_mode(
            &cfg,
            &root,
            &destinations,
            root.clone(),
            Scope::Project,
            true,
            &LockMode::Plain,
        );
        let mut updated = UpdatedAt::default();
        let result = sync(&ctx, &mut lock, &mut updated);
        assert_eq!(result.summary.failed, 0, "{:?}", result.actions);
        assert_eq!(result.summary.installed, 0, "{:?}", result.actions);
        assert_eq!(result.summary.updated, 1, "{:?}", result.actions);
        assert_eq!(result.actions[0].status, "updated");
        assert_eq!(lock.version, LOCK_VERSION);
        assert_eq!(lock.installed_outputs.len(), 3);
        for skill_root in &destinations {
            let destination = skill_root.join("alpha");
            assert_eq!(
                fs::metadata(&destination).unwrap().permissions().mode() & 0o777,
                0o755
            );
            assert_eq!(
                fs::metadata(destination.join("SKILL.md"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o644
            );
        }
        assert_eq!(
            fs::read_to_string(hand_authored_harness.join("SKILL.md")).unwrap(),
            "hand-authored; agent-env must preserve this tree"
        );
        let _ = crate::runtime::clear_runtime_state(Scope::Project, &root);
        let _ = fs::remove_dir_all(root);
    }
}
