//! db_index — scalable file index over repo/control-plane files (REQ-052).
//!
//! REQ-050 scaffold: the [`DbFileRow`] shape and the [`FileIndex`] seam. The
//! real walk (respecting `.gitignore` via `ignore::WalkBuilder`, targetable to
//! narrow scopes, content-hashed) lands in REQ-052; `watch` in REQ-057.

use crate::db::{MutablePolicy, Result};
use serde::{Deserialize, Serialize};

/// One indexed file. `mutable_policy`/`protected`/`generated` drive what the
/// refactor and deploy planners are allowed to touch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbFileRow {
    pub file_id: String,
    pub absolute_path: String,
    pub repo_relative_path: Option<String>,
    pub logical_owner: Option<String>,
    pub file_kind: String,
    pub parser_hint: String,
    pub content_hash: String,
    pub byte_len: u64,
    pub line_count: usize,
    pub generated: bool,
    pub protected: bool,
    pub mutable_policy: MutablePolicy,
    pub last_indexed_at: String,
}

/// Options bounding a scan — never walk giant unrelated trees by default.
#[derive(Debug, Clone, Default)]
pub struct ScanScope {
    /// Root to scan (repo root or a narrow subdir).
    pub root: String,
    /// Optional extra roots to include when explicitly requested.
    pub extra_roots: Vec<String>,
    /// Respect `.gitignore` (default true in REQ-052).
    pub respect_gitignore: bool,
}

/// The file index. REQ-050 provides the container + empty seam.
#[derive(Debug, Clone, Default)]
pub struct FileIndex {
    files: Vec<DbFileRow>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn files(&self) -> &[DbFileRow] {
        &self.files
    }

    /// Build the index from a scope. REQ-052 implements the `ignore::WalkBuilder`
    /// parallel walk + content hashing; the scaffold returns an empty index so
    /// callers compile and tests can assert on shape.
    pub fn scan(_scope: &ScanScope) -> Result<Self> {
        Ok(Self::default())
    }
}
