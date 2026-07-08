//! db_query — deterministic query surface + agent presets (REQ-054).
//!
//! REQ-050 scaffold: the query AST ([`QueryTable`], [`QueryFilter`],
//! [`QuerySpec`]), the preset enum, and the [`QueryResult`] shape. A minimal,
//! deterministic evaluator (no SQL clone) + `--explain` land in REQ-054.

use crate::db::Result;
use serde::{Deserialize, Serialize};

/// Selectable tables in the query surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryTable {
    Files,
    Symbols,
    Occurrences,
    Roots,
    Refs,
    Actions,
}

/// A single deterministic filter clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum QueryFilter {
    Eq { field: String, value: String },
    Contains { field: String, value: String },
    In { field: String, values: Vec<String> },
    PathMatches { glob: String },
}

/// Agent-facing preset queries (stable names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueryPreset {
    RootMeta,
    RootLifeos,
    HooksCodex,
    WrappersBroken,
    MutableUnsafe,
    SymbolsRustCli,
    PathsLegacy,
}

/// A resolved query: either a table+filters form or a preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySpec {
    pub table: Option<QueryTable>,
    pub filters: Vec<QueryFilter>,
    pub preset: Option<QueryPreset>,
    pub target_profile: Option<String>,
    /// When true, the result carries an [`QueryResult::explain`] trace of the
    /// tables/filters used (the `--explain` contract).
    pub explain: bool,
}

/// Query output — rows are JSON values so the surface stays table-agnostic and
/// the `--json` machine contract (REQ-058) is stable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryResult {
    pub rows: Vec<serde_json::Value>,
    pub row_count: usize,
    /// Populated when `explain` was requested: which tables/filters ran.
    pub explain: Option<String>,
}

/// Evaluate a query against the indexes. REQ-054 implements the deterministic
/// evaluator + presets; the scaffold returns an empty, well-formed result.
pub fn evaluate(
    _spec: &QuerySpec,
    _files: &crate::db_index::FileIndex,
    _symbols: &crate::db_symbols::SymbolIndex,
) -> Result<QueryResult> {
    Ok(QueryResult::default())
}
