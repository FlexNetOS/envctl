//! db_refactor — root-alias refactor planner (REQ-055).
//!
//! REQ-050 scaffold: the [`RootAliasSpec`] input, the [`RefactorPlan`] /
//! [`RefactorChange`] output, and the [`plan`] seam. Behaviour contract
//! (implemented in REQ-055): build an indexed snapshot, resolve root aliases +
//! token forms, emit a `similar`-diff preview, `--render-out` writes a NEW tree
//! (never in place), and `--apply` requires `--confirm` + an approved approval.

use crate::db::Result;
use serde::{Deserialize, Serialize};

/// Request to re-point one root variable to another across a scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAliasSpec {
    /// e.g. `META_ROOT`.
    pub from: String,
    /// e.g. `LIFE_OS_ROOT`.
    pub to: String,
    /// e.g. `lifeos-release`.
    pub target_profile: Option<String>,
    /// Restrict the rewrite to a scope (path/preset); empty means whole repo.
    pub scope: Option<String>,
    /// When set, write the rewritten tree here instead of in place.
    pub render_out: Option<String>,
}

/// How the plan may be executed. Fail-closed default is [`ApplyMode::Plan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApplyMode {
    #[default]
    Plan,
    /// Write a new tree at `render_out`; originals untouched.
    Render,
    /// Mutate in place — requires confirm + approval (R3).
    Apply,
}

/// A single proposed change (unified-diff preview carried as text).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefactorChange {
    pub file_id: String,
    pub absolute_path: String,
    pub occurrence_count: usize,
    /// `similar`-style unified diff preview (REQ-055).
    pub unified_diff: String,
    pub safe: bool,
}

/// The plan the CLI/GUI render and the approval gate reasons over.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefactorPlan {
    pub mode: ApplyMode,
    pub changes: Vec<RefactorChange>,
    pub files_touched: usize,
    pub occurrences_total: usize,
    /// Occurrences the planner refuses to auto-rewrite (needs owner/parser).
    pub refused: usize,
    /// True when `mode == Apply` and confirm+approval were both supplied.
    pub approved: bool,
}

/// Build a refactor plan. REQ-055 implements resolution + diffing + render/apply
/// discipline; the scaffold returns an empty plan in the fail-closed `Plan`
/// mode so nothing can mutate through the seam.
pub fn plan(
    _spec: &RootAliasSpec,
    _files: &crate::db_index::FileIndex,
    _symbols: &crate::db_symbols::SymbolIndex,
) -> Result<RefactorPlan> {
    Ok(RefactorPlan::default())
}
