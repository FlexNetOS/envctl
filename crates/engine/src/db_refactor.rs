//! db_refactor — root-alias refactor planner (REQ-055).
//!
//! Re-points one root variable to another (e.g. `META_ROOT` → `LIFE_OS_ROOT`)
//! across an indexed snapshot. The rewrite is *normalization-aware*, not a blind
//! `$META_ROOT` string replace (REQ-051 rule): a token is rewritten only when its
//! name normalizes to the `from` root, so the `LIFEOS_ROOT` alias is caught and
//! ordinary identifiers like `$META_ROOT_FALLBACK` are not.
//!
//! Discipline (R3 / `human_approval_required`):
//! - [`plan`] builds a fail-closed [`ApplyMode::Plan`] plan with a per-line
//!   unified-diff preview; it never touches the filesystem.
//! - [`render`] writes the rewritten tree to `spec.render_out` (atomic
//!   temp+rename); originals are never modified.
//! - [`apply`] mutates in place *only* when `confirm == true` AND an approved
//!   [`Approval`] is supplied — otherwise it returns [`DbError::RefactorBlocked`].
//!
//! Occurrences in refuse-policy files (`.env`, secrets, …) are counted in
//! [`RefactorPlan::refused`] and never rewritten.

use crate::db::{normalize_root_var, DbError, Result};
use crate::db_index::FileIndex;
use crate::db_symbols::{is_var_name, SymbolIndex};
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
    pub repo_relative_path: Option<String>,
    pub occurrence_count: usize,
    /// Per-line unified diff preview (REQ-055).
    pub unified_diff: String,
    pub safe: bool,
    /// Why an unsafe change was refused (empty when `safe`).
    pub refused_reason: String,
}

/// The plan the CLI/GUI render and the approval gate reasons over.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefactorPlan {
    pub mode: ApplyMode,
    pub changes: Vec<RefactorChange>,
    pub files_touched: usize,
    pub occurrences_total: usize,
    /// Occurrences the planner refuses to auto-rewrite (protected / needs owner).
    pub refused: usize,
    /// True when `mode == Apply` and confirm+approval were both supplied.
    pub approved: bool,
}

/// A human approval record for an [`ApplyMode::Apply`] execution (R3 gate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub approver: String,
    pub approved: bool,
    pub note: Option<String>,
}

/// Build a refactor plan: find every occurrence whose name normalizes to
/// `spec.from`, group by file, and emit a per-line unified diff for the files
/// whose policy allows a safe rewrite. Protected/needs-owner files are surfaced
/// as refused changes (never rewritten). Fail-closed [`ApplyMode::Plan`]; the
/// filesystem is not touched.
pub fn plan(
    spec: &RootAliasSpec,
    files: &FileIndex,
    symbols: &SymbolIndex,
) -> Result<RefactorPlan> {
    let from_norm = normalize_root_var(&spec.from);
    let to_norm = normalize_root_var(&spec.to);
    if from_norm == to_norm {
        return Err(DbError::RefactorBlocked(format!(
            "from and to resolve to the same root: {from_norm}"
        )));
    }

    // File ids carrying a `from` occurrence (deterministic order).
    let mut file_ids: Vec<&str> = symbols
        .occurrences()
        .iter()
        .filter(|o| o.normalized_text == from_norm)
        .map(|o| o.file_id.as_str())
        .collect();
    file_ids.sort_unstable();
    file_ids.dedup();

    let mut changes = Vec::new();
    let mut occurrences_total = 0usize;
    let mut refused = 0usize;
    let mut files_touched = 0usize;

    for fid in file_ids {
        let file = match files.files().iter().find(|f| f.file_id == fid) {
            Some(f) => f,
            None => continue,
        };
        // Occurrences of `from` in this file share a replace_policy (it derives
        // from the file's mutable_policy), so the file is wholly safe or refused.
        let occs: Vec<_> = symbols
            .occurrences()
            .iter()
            .filter(|o| o.file_id == fid && o.normalized_text == from_norm)
            .collect();
        let count = occs.len();
        occurrences_total += count;
        let safe = occs.iter().all(|o| o.replace_candidate);
        let rel = file.repo_relative_path.clone();

        if !safe {
            refused += count;
            changes.push(RefactorChange {
                file_id: fid.into(),
                absolute_path: file.absolute_path.clone(),
                repo_relative_path: rel,
                occurrence_count: count,
                unified_diff: String::new(),
                safe: false,
                refused_reason: format!("policy {:?} refuses auto-rewrite", file.mutable_policy),
            });
            continue;
        }

        let original = match std::fs::read_to_string(&file.absolute_path) {
            Ok(c) => c,
            Err(e) => {
                refused += count;
                changes.push(RefactorChange {
                    file_id: fid.into(),
                    absolute_path: file.absolute_path.clone(),
                    repo_relative_path: rel,
                    occurrence_count: count,
                    unified_diff: String::new(),
                    safe: false,
                    refused_reason: format!("unreadable: {e}"),
                });
                continue;
            }
        };
        let (rewritten, replaced) = rewrite_env_tokens(&original, &from_norm, &to_norm);
        if replaced == 0 || rewritten == original {
            continue;
        }
        let diff_rel = rel.clone().unwrap_or_else(|| file.absolute_path.clone());
        files_touched += 1;
        changes.push(RefactorChange {
            file_id: fid.into(),
            absolute_path: file.absolute_path.clone(),
            repo_relative_path: rel,
            occurrence_count: replaced,
            unified_diff: unified_diff(&diff_rel, &original, &rewritten),
            safe: true,
            refused_reason: String::new(),
        });
    }

    changes.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));
    Ok(RefactorPlan {
        mode: ApplyMode::Plan,
        changes,
        files_touched,
        occurrences_total,
        refused,
        approved: false,
    })
}

/// Render the plan's safe changes into a NEW tree rooted at `spec.render_out`,
/// preserving each file's repo-relative path. Originals are never touched. Each
/// file is written atomically (temp + rename). Returns the paths written.
pub fn render(plan: &RefactorPlan, spec: &RootAliasSpec, files: &FileIndex) -> Result<Vec<String>> {
    let out_root = spec
        .render_out
        .as_ref()
        .ok_or_else(|| DbError::RefactorBlocked("render requires spec.render_out".into()))?;
    let from_norm = normalize_root_var(&spec.from);
    let to_norm = normalize_root_var(&spec.to);
    let out_base = std::path::Path::new(out_root);
    let mut written = Vec::new();

    for change in plan.changes.iter().filter(|c| c.safe) {
        let file = files
            .files()
            .iter()
            .find(|f| f.file_id == change.file_id)
            .ok_or_else(|| DbError::RefactorBlocked(format!("file gone: {}", change.file_id)))?;
        let original = std::fs::read_to_string(&file.absolute_path)?;
        let (rewritten, _) = rewrite_env_tokens(&original, &from_norm, &to_norm);
        let rel = change
            .repo_relative_path
            .clone()
            .unwrap_or_else(|| sanitize_rel(&file.absolute_path));
        let dest = out_base.join(&rel);
        atomic_write(&dest, rewritten.as_bytes())?;
        written.push(dest.display().to_string());
    }
    written.sort();
    Ok(written)
}

/// Apply the plan's safe changes IN PLACE — the destructive path. Fail-closed:
/// requires `confirm == true` AND an approved [`Approval`]. Each file is written
/// atomically (temp + rename). Returns the paths mutated.
pub fn apply(
    plan: &RefactorPlan,
    spec: &RootAliasSpec,
    files: &FileIndex,
    confirm: bool,
    approval: Option<&Approval>,
) -> Result<Vec<String>> {
    if !confirm {
        return Err(DbError::RefactorBlocked(
            "apply requires --confirm (R3): refusing in-place rewrite".into(),
        ));
    }
    match approval {
        Some(a) if a.approved => {}
        _ => {
            return Err(DbError::RefactorBlocked(
                "apply requires an approved approval (R3/human_approval_required)".into(),
            ))
        }
    }
    let from_norm = normalize_root_var(&spec.from);
    let to_norm = normalize_root_var(&spec.to);
    let mut mutated = Vec::new();
    for change in plan.changes.iter().filter(|c| c.safe) {
        let file = files
            .files()
            .iter()
            .find(|f| f.file_id == change.file_id)
            .ok_or_else(|| DbError::RefactorBlocked(format!("file gone: {}", change.file_id)))?;
        let original = std::fs::read_to_string(&file.absolute_path)?;
        let (rewritten, _) = rewrite_env_tokens(&original, &from_norm, &to_norm);
        atomic_write(
            std::path::Path::new(&file.absolute_path),
            rewritten.as_bytes(),
        )?;
        mutated.push(file.absolute_path.clone());
    }
    mutated.sort();
    Ok(mutated)
}

/// Normalization-aware token rewrite: replaces `$VAR`/`${VAR}` with the canonical
/// `to` form when `VAR` normalizes to `from_norm` — preserving bracket style and
/// leaving unrelated identifiers untouched. Returns (rewritten, replacements).
fn rewrite_env_tokens(content: &str, from_norm: &str, to_norm: &str) -> (String, usize) {
    let bytes = content.as_bytes();
    let mut out = String::with_capacity(content.len());
    let mut last = 0usize;
    let mut i = 0usize;
    let mut replaced = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let matched: Option<(usize, String)> = if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                let start = i + 2;
                let mut j = start;
                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }
                let name = (j < bytes.len()).then(|| &content[start..j]);
                match name {
                    Some(n) if is_var_name(n) && normalize_root_var(n) == from_norm => {
                        Some((j + 1, format!("${{{to_norm}}}")))
                    }
                    _ => None,
                }
            } else {
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                let name = (j > start).then(|| &content[start..j]);
                match name {
                    Some(n) if is_var_name(n) && normalize_root_var(n) == from_norm => {
                        Some((j, format!("${to_norm}")))
                    }
                    _ => None,
                }
            };
            if let Some((end, repl)) = matched {
                out.push_str(&content[last..i]);
                out.push_str(&repl);
                last = end;
                i = end;
                replaced += 1;
                continue;
            }
        }
        i += 1;
    }
    out.push_str(&content[last..]);
    (out, replaced)
}

/// Minimal deterministic per-line unified diff. Inline token rewrites keep the
/// line count stable, so old/new lines pair 1:1; only differing lines are shown.
fn unified_diff(rel: &str, old: &str, new: &str) -> String {
    let mut out = format!("--- a/{rel}\n+++ b/{rel}\n");
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    for (idx, (o, n)) in old_lines.iter().zip(new_lines.iter()).enumerate() {
        if o != n {
            out.push_str(&format!("@@ line {} @@\n-{o}\n+{n}\n", idx + 1));
        }
    }
    out
}

/// Turn an absolute path into a safe relative path under a render root.
fn sanitize_rel(abs: &str) -> String {
    abs.trim_start_matches('/').replace("..", "__")
}

/// Write `bytes` to `dest` atomically: write a sibling temp file, then rename.
/// Creates parent dirs. Never leaves a half-written destination.
fn atomic_write(dest: &std::path::Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension(format!(
        "envctl-tmp-{}",
        dest.extension().and_then(|e| e.to_str()).unwrap_or("out")
    ));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_index::ScanScope;
    use std::fs;

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d =
            std::env::temp_dir().join(format!("envctl-db-refactor-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn indexes(root: &std::path::Path) -> (FileIndex, SymbolIndex) {
        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();
        (files, symbols)
    }

    fn meta_spec(render_out: Option<String>) -> RootAliasSpec {
        RootAliasSpec {
            from: "META_ROOT".into(),
            to: "LIFE_OS_ROOT".into(),
            target_profile: Some("lifeos-release".into()),
            scope: None,
            render_out,
        }
    }

    #[test]
    fn plan_emits_diff_for_safe_files_and_refuses_protected() {
        let root = tmp("plan");
        // Shell wrapper (OwnedApply -> Safe). Includes an alias + a non-target
        // token that must NOT be rewritten.
        fs::write(
            root.join("wrapper.sh"),
            b"cd \"$META_ROOT/bin\"\nexport A=${META_ROOT}/x\nexport KEEP=$META_ROOT_FALLBACK\n",
        )
        .unwrap();
        // Protected .env (Never -> Refuse): must be surfaced, never rewritten.
        fs::write(root.join(".env"), b"SECRET=$META_ROOT/secrets\n").unwrap();

        let (files, symbols) = indexes(&root);
        let p = plan(&meta_spec(None), &files, &symbols).unwrap();

        assert_eq!(p.mode, ApplyMode::Plan);
        assert!(!p.approved);
        assert_eq!(p.files_touched, 1, "only wrapper.sh is safely rewritable");

        let wrapper = p
            .changes
            .iter()
            .find(|c| c.absolute_path.ends_with("wrapper.sh"))
            .expect("wrapper change");
        assert!(wrapper.safe);
        // Two real occurrences ($META_ROOT and ${META_ROOT}); the _FALLBACK token
        // is a different name and is left alone.
        assert_eq!(wrapper.occurrence_count, 2);
        assert!(wrapper.unified_diff.contains("+cd \"$LIFE_OS_ROOT/bin\""));
        assert!(wrapper.unified_diff.contains("+export A=${LIFE_OS_ROOT}/x"));
        assert!(
            !wrapper.unified_diff.contains("KEEP"),
            "non-target token must not appear in the diff"
        );

        // .env is refused, not rewritten.
        let env = p
            .changes
            .iter()
            .find(|c| c.absolute_path.ends_with(".env"))
            .expect("env change surfaced");
        assert!(!env.safe);
        assert!(env.unified_diff.is_empty());
        assert!(p.refused >= 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn render_writes_new_tree_never_in_place_and_normalizes_alias() {
        let root = tmp("render");
        // Alias spelling: $LIFEOS_ROOT must be caught when from == LIFE_OS_ROOT,
        // proving normalization-aware rewrite (not a blind string match).
        let original = b"cd $META_ROOT\nalias_line=${LIFEOS_ROOT}\n";
        fs::write(root.join("wrapper.sh"), original).unwrap();

        let (files, symbols) = indexes(&root);
        let out = tmp("render-out");
        let spec = RootAliasSpec {
            from: "META_ROOT".into(),
            to: "LIFE_OS_ROOT".into(),
            target_profile: None,
            scope: None,
            render_out: Some(out.display().to_string()),
        };
        let p = plan(&spec, &files, &symbols).unwrap();
        let written = render(&p, &spec, &files).unwrap();

        assert_eq!(written.len(), 1);
        let rendered = fs::read_to_string(&written[0]).unwrap();
        assert!(rendered.contains("cd $LIFE_OS_ROOT"));
        // Original on disk is untouched.
        let still = fs::read_to_string(root.join("wrapper.sh")).unwrap();
        assert_eq!(still.as_bytes(), original, "originals must never change");

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn apply_is_fail_closed_without_confirm_and_approval() {
        let root = tmp("apply");
        fs::write(root.join("wrapper.sh"), b"cd $META_ROOT/bin\n").unwrap();
        let (files, symbols) = indexes(&root);
        let spec = meta_spec(None);
        let p = plan(&spec, &files, &symbols).unwrap();

        // No confirm -> blocked.
        assert!(apply(&p, &spec, &files, false, None).is_err());
        // Confirm but no approval -> blocked.
        assert!(apply(&p, &spec, &files, true, None).is_err());
        // Confirm + un-approved approval -> blocked.
        let denied = Approval {
            approver: "op".into(),
            approved: false,
            note: None,
        };
        assert!(apply(&p, &spec, &files, true, Some(&denied)).is_err());
        // Original still pristine after all refusals.
        assert_eq!(
            fs::read_to_string(root.join("wrapper.sh")).unwrap(),
            "cd $META_ROOT/bin\n"
        );

        // Confirm + approved -> applies in place, atomically.
        let ok = Approval {
            approver: "op".into(),
            approved: true,
            note: Some("REQ-055 gate".into()),
        };
        let mutated = apply(&p, &spec, &files, true, Some(&ok)).unwrap();
        assert_eq!(mutated.len(), 1);
        assert_eq!(
            fs::read_to_string(root.join("wrapper.sh")).unwrap(),
            "cd $LIFE_OS_ROOT/bin\n"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn plan_refuses_identity_rewrite() {
        let root = tmp("identity");
        fs::write(root.join("w.sh"), b"cd $META_ROOT\n").unwrap();
        let (files, symbols) = indexes(&root);
        let spec = RootAliasSpec {
            from: "LIFEOS_ROOT".into(),
            to: "LIFE_OS_ROOT".into(), // both normalize to LIFE_OS_ROOT
            target_profile: None,
            scope: None,
            render_out: None,
        };
        assert!(plan(&spec, &files, &symbols).is_err());
        let _ = fs::remove_dir_all(&root);
    }
}
