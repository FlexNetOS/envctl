//! `envctl-agent-env` — the pure-Rust agent-environment provisioning core.
//!
//! This crate is the foundational, **standalone** library of the kasetto absorption
//! (Epic C, TASK-0012): the config model + `extends` inheritance + multi-host source
//! resolver + versioned SHA-256 content hashing + the agent-asset lock (with its 3
//! modes). It is ported faithfully from the live `kasetto` **v3.2.0** source with NO
//! capability downgrade (see `.handoff/decisions/ADR-0001`).
//!
//! Scope boundaries (deliberately deferred to later cards):
//! - This crate is a **library only**: non-printing, no `clap`, no UI, no `Engine`. It
//!   returns `Result<T, AgentEnvError>` and typed data; callers (TASK-0013 Engine,
//!   TASK-0014 CLI/GUI) drive it.
//! - The `Agent` enum models the full 21-preset **shape** plus the per-agent native
//!   path-mapping methods (skills/mcps/commands destinations) and their MCP/command
//!   format+target value types (see [`agent`] and [`report`]). The sync *engine* that
//!   drives them — `commands/sync`, `commands/list` — is deferred to TASK-0013.
//! - The agent-asset lock uses **SHA-256** and is a **separate** type from the engine's
//!   FNV-1a component lock (`crates/engine/src/lock.rs`) — they do not share code.
//!
//! Invariants upheld: no C in the trust boundary (pure-Rust gzip via miniz_oxide, pure-Rust
//! tar, sha2, reqwest→rustls→ring), fail-closed guards (tar-slip path-traversal refusal,
//! `--locked` zero-network), and `#![forbid(unsafe_code)]` (set via `[lints]`).

pub mod agent;
pub mod command;
pub mod config;
pub mod config_edit;
pub mod config_path;
pub mod dirs;
pub mod driver;
pub mod extend;
pub mod fsops;
pub mod hash;
pub mod lock;
pub mod mcp;
pub mod profile;
pub mod report;
pub mod runtime;
pub mod runtime_contract;
mod secure_file;
pub mod source;
pub mod sync;
pub mod tree;
pub mod util;

/// Atomically replace an authoritative agent-env config without following a leaf/parent symlink.
/// Existing stricter permissions are preserved; permissive modes are clamped to project-safe
/// `0644` semantics on Unix.
pub fn write_config_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    let path = absolute_local_path(path)?;
    secure_file::write_atomic(&path, bytes, 0o644)
}

/// Create a new authoritative config, or replace an existing regular config when `force` is set.
/// The create-only path uses an atomic no-clobber commit, so an object appearing after the
/// no-follow authority check is refused rather than overwritten. Returns whether a regular file
/// was replaced.
pub fn initialize_config_atomic(path: &std::path::Path, bytes: &[u8], force: bool) -> Result<bool> {
    let path = absolute_local_path(path)?;
    secure_file::initialize_atomic(&path, bytes, 0o644, force)
}

/// Read an optional authoritative agent-env config without following a leaf or parent symlink.
pub fn read_config_optional(path: &std::path::Path) -> Result<Option<Vec<u8>>> {
    let path = absolute_local_path(path)?;
    secure_file::read_optional(&path)
}

fn absolute_local_path(path: &std::path::Path) -> Result<std::path::PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

/// Read-only safety check for the existing authority chain that would own a managed path.
pub fn managed_path_authority_is_safe(path: &std::path::Path) -> bool {
    let authority = match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => path,
        _ => match path.parent() {
            Some(parent) => parent,
            None => return false,
        },
    };
    secure_file::validate_parent_chain(authority).is_ok()
}

pub use agent::{
    all_command_global_targets, all_command_project_targets, all_mcp_project_targets,
    all_mcp_settings_targets, command_global_targets, command_project_targets, CommandFormat,
    CommandTarget, McpSettingsFormat, McpSettingsTarget,
};
pub use command::{
    apply_command, apply_command_text_atomic, command_destination_matches_text, destination_path,
    ensure_parent_dirs, parse, render, validate_command_name, Parsed,
};
pub use config::{
    git_pin_of, Agent, AgentField, CommandEntry, CommandSourceSpec, CommandsField, Config, GitPin,
    McpEntry, McpSourceSpec, McpsField, Scope, SkillTarget, SkillsField, SourceSpec, AGENT_PRESETS,
};
pub use config_edit::{
    ensure_local_config, insert_item, is_remote_source, item_exists, remove_item, remove_names,
    split_at_ref, Pin, RemoveOutcome, Section, Selector, SourceItem,
};
pub use config_path::{
    default_config_path, resolve_config_path, Preferences, CONFIG_ENV_VAR, DEFAULT_CONFIG_FILENAME,
    DEFAULT_GLOBAL_CONFIG_FILENAME, PREFERENCES_FILENAME,
};
pub use dirs::{
    dirs_agent_env_cache, dirs_agent_env_config, dirs_agent_env_data, dirs_home,
    dirs_xdg_cache_home, dirs_xdg_config_home, dirs_xdg_data_home,
};
pub use driver::{
    clean_actions, clean_apply_transaction, inspect_installed_inventory, load_skills_mcps_commands,
    plan_add_edits, rebuild_lock, sync, verify_source, AssetRow, CleanCounts, DriverCtx,
    InstalledInventory, SectionEdit, SyncResult, UpdatedAt,
};
pub use extend::{
    extract_extends, load_config_any, load_config_any_zero_network, load_config_recursive,
    merge_yaml, MAX_EXTENDS_DEPTH,
};
pub use fsops::{
    copy_dir, copy_dir_contents, copy_file, relativize_dest, resolve_command_targets, resolve_dest,
    resolve_destinations, resolve_mcp_settings_targets, resolve_path, scope_root, select_targets,
    BrokenSkill, SettingsFile, TargetSelection,
};
pub use hash::{hash_bytes, hash_dir, hash_file, hash_str};
pub use lock::{
    AgentLockEntry, AgentLockFile, AgentLockState, AssetEntry, LockMode, AGENT_ASSETS_KEY,
    LOCK_VERSION,
};
pub use mcp::{
    apply_mcp_config_bytes_atomic, conflicting_mcp_server_names, current_mcp_fragment_hashes,
    expected_mcp_fragment_hashes, mcp_server_names_from_bytes, merge_mcp_config,
    read_source_mcp_servers, read_source_mcp_servers_bytes, remove_mcp_server,
    render_mcp_settings_bytes, servers_present_in_settings,
};
pub use profile::{format_updated_ago, read_skill_profile, read_skill_profile_from_dir};
pub use report::{Action, InstalledSkill, Report, Summary, SyncFailure};
pub use runtime::{
    clear_runtime_state, load_runtime_state, runtime_state_path, save_runtime_state, ManagedOutput,
    RuntimeState,
};
pub use runtime_contract::validate_runtime_contract;
pub use source::{
    archive_url, derive_browse_url, discover, discover_commands, discover_mcps,
    discover_with_root_name, download_extract, materialize_source, parse_repo_url,
    resolve_command_entry, resolve_mcp_entry, rewrite_browse_to_raw_url, BrowseDerived,
    MaterializedSource, RepoUrl, UrlRequestAuth,
};
pub use sync::{
    command_action_label, command_asset_id, mcp_action_label, mcp_asset_id, remove_stale,
    skill_key, StaleEntry,
};
pub use tree::TreeSnapshot;
pub use util::{now_unix, now_unix_str};

/// Result alias for the agent-env crate.
pub type Result<T> = std::result::Result<T, AgentEnvError>;

/// Typed error for the agent-env crate.
///
/// Faithful ports of kasetto's string-message `err(...)` calls land on
/// [`AgentEnvError::Message`]; structured failures use the dedicated variants.
#[derive(Debug, thiserror::Error)]
pub enum AgentEnvError {
    /// A free-form failure message (the absorbed kasetto `err(...)` channel).
    #[error("{0}")]
    Message(String),

    /// An I/O failure (file read/write, archive extraction).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A YAML (de)serialization failure for config or lock files.
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// A JSON (de)serialization failure for agent settings files (the `SettingsFile`
    /// save/merge path; mirrors kasetto's `?`-propagated `serde_json::Error`).
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Construct an [`AgentEnvError::Message`] — the string-message error channel that
/// mirrors kasetto's `err(...)` helper, so the ported control flow is line-for-line.
pub(crate) fn err(message: impl Into<String>) -> AgentEnvError {
    AgentEnvError::Message(message.into())
}

#[cfg(test)]
mod xc01_error_channel_tests {
    //! XC-01 parity — the string-message error channel.
    //!
    //! Oracle: kasetto `src/error.rs` — `pub(crate) fn err(message: impl Into<String>) ->
    //! Error` where `Error = Box<dyn std::error::Error + Send + Sync>`, built via
    //! `std::io::Error::other(message.into()).into()`, plus `pub type Result<T> =
    //! std::result::Result<T, Error>`.
    //!
    //! Idiom map (kasetto → envctl): kasetto's boxed-`dyn Error` channel built from
    //! `io::Error::other` is ported to a typed `thiserror` enum whose
    //! [`AgentEnvError::Message`] variant carries the free-form string. Both share the same
    //! **observable** contract that this test pins: a string message handed to `err(...)`
    //! round-trips byte-for-byte through the error's `Display`, and the `Result<T>` alias
    //! resolves to the crate error. `err()` is `pub(crate)`, so this lives in-crate.
    use super::{err, AgentEnvError, Result};

    #[test]
    fn err_produces_message_variant_whose_display_contains_the_message() {
        let e = err("boom: something went wrong");
        // Lands on the Message arm (the absorbed kasetto string-`err` channel), not Io/Yaml/Json.
        assert!(
            matches!(e, AgentEnvError::Message(_)),
            "err(...) must land on the Message variant, got {e:?}"
        );
        // Observable parity with kasetto: the string round-trips through Display verbatim.
        assert_eq!(e.to_string(), "boom: something went wrong");
    }

    #[test]
    fn err_accepts_both_str_and_string_via_into() {
        // Mirrors kasetto `err(impl Into<String>)`: &str and String both flow through.
        let from_str = err("msg");
        let from_string = err(String::from("msg"));
        assert!(from_str.to_string().contains("msg"));
        assert!(from_string.to_string().contains("msg"));
    }

    #[test]
    fn result_alias_resolves_to_crate_error() {
        // The `Result<T>` alias resolves to `Result<T, AgentEnvError>` (kasetto: `Result<T,
        // Box<dyn Error + Send + Sync>>`). Exercise both arms so the alias is type-checked.
        fn ok_path() -> Result<u32> {
            Ok(7)
        }
        fn err_path() -> Result<u32> {
            Err(err("nope"))
        }
        assert_eq!(ok_path().unwrap(), 7);
        assert_eq!(err_path().unwrap_err().to_string(), "nope");
    }
}
