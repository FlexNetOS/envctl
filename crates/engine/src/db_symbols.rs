//! db_symbols — symbol + occurrence index over indexed files (REQ-053).
//!
//! REQ-050 scaffold: the [`DbSymbolKind`], [`DbSymbolRow`], and
//! [`DbOccurrenceRow`] shapes plus the [`SymbolIndex`] seam. Rust symbols come
//! from `syn` in-core; polyglot structural matching (ast-grep/tree-sitter) is
//! wired as an external managed component so the no-C gate holds (REQ-053/060).

use crate::db::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbSymbolKind {
    EnvVar,
    PathToken,
    RustItem,
    CliSubcommand,
    HookScript,
    WrapperScript,
    ConfigKey,
    ComponentId,
    RegistryEntry,
    AgentAsset,
    SecretReference,
    Unknown,
}

/// How the symbol was resolved — drives whether a rewrite is safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolConfidence {
    Exact,
    Parsed,
    Heuristic,
    ExternalTool,
}

/// Whether an occurrence can be mechanically rewritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplacePolicy {
    Safe,
    NeedsParser,
    NeedsOwnerMarker,
    Refuse,
    ManualReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbSymbolRow {
    pub symbol_id: String,
    pub kind: DbSymbolKind,
    pub name: String,
    pub normalized_name: String,
    pub file_id: String,
    pub absolute_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub value: Option<String>,
    pub scope: Option<String>,
    pub owner_component: Option<String>,
    pub target_profile: Option<String>,
    pub confidence: SymbolConfidence,
    pub mutable_policy: crate::db::MutablePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbOccurrenceRow {
    pub occurrence_id: String,
    pub symbol_id: String,
    pub file_id: String,
    pub match_text: String,
    pub normalized_text: String,
    pub line: usize,
    pub column: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub context_before: String,
    pub context_after: String,
    pub replace_candidate: bool,
    pub replace_policy: ReplacePolicy,
}

/// The symbol/occurrence index. REQ-050 provides the container + empty seam.
#[derive(Debug, Clone, Default)]
pub struct SymbolIndex {
    symbols: Vec<DbSymbolRow>,
    occurrences: Vec<DbOccurrenceRow>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbols(&self) -> &[DbSymbolRow] {
        &self.symbols
    }

    pub fn occurrences(&self) -> &[DbOccurrenceRow] {
        &self.occurrences
    }

    /// Build the symbol/occurrence index from an already-built file index.
    /// REQ-053 implements env-var/path-token/Rust-item extraction; the scaffold
    /// returns an empty index.
    pub fn build(_files: &crate::db_index::FileIndex) -> Result<Self> {
        Ok(Self::default())
    }
}
