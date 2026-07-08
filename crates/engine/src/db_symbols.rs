//! db_symbols — symbol + occurrence index over indexed files (REQ-053).
//!
//! REQ-050 scaffold: the [`DbSymbolKind`], [`DbSymbolRow`], and
//! [`DbOccurrenceRow`] shapes plus the [`SymbolIndex`] seam. Rust symbols come
//! from `syn` in-core; polyglot structural matching (ast-grep/tree-sitter) is
//! wired as an external managed component so the no-C gate holds (REQ-053/060).

use crate::db::{normalize_root_var, MutablePolicy, Result};
use crate::db_index::{DbFileRow, FileIndex};
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

    /// Build the symbol/occurrence index from an already-built file index by
    /// scanning each file's content for environment-variable / path-token
    /// references of the form `$VAR`, `${VAR}`, and bare `UPPER_SNAKE` roots.
    ///
    /// This is a byte/line scan (no `syn`/tree-sitter dependency — the no-C
    /// boundary stays trivially intact; full Rust-item extraction via `syn` is a
    /// REQ-060-gated dependency decision). Each occurrence's [`ReplacePolicy`] is
    /// derived from the owning file's [`MutablePolicy`], so the refactor planner
    /// (REQ-055) never proposes an unsafe rewrite.
    pub fn build(files: &FileIndex) -> Result<Self> {
        let mut idx = Self::default();
        for file in files.files() {
            let content = match std::fs::read_to_string(&file.absolute_path) {
                Ok(c) => c,
                Err(_) => continue, // binary / unreadable — skip, don't fail the build
            };
            idx.scan_file(file, &content);
        }
        idx.symbols.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));
        idx.occurrences
            .sort_by(|a, b| a.occurrence_id.cmp(&b.occurrence_id));
        Ok(idx)
    }

    /// Scan one file's content, appending symbols (deduped by normalized name)
    /// and one occurrence per hit.
    fn scan_file(&mut self, file: &DbFileRow, content: &str) {
        let replace_policy = replace_policy_for(file.mutable_policy);
        for (line_no, line) in content.lines().enumerate() {
            for hit in scan_line_env_refs(line) {
                let normalized = normalize_root_var(&hit.name);
                let kind = classify_symbol(&normalized, &file.file_kind);
                let symbol_id = format!("sym:{}:{}", kind_tag(&kind), normalized);
                if !self.symbols.iter().any(|s| s.symbol_id == symbol_id) {
                    self.symbols.push(DbSymbolRow {
                        symbol_id: symbol_id.clone(),
                        kind: kind.clone(),
                        name: hit.name.clone(),
                        normalized_name: normalized.clone(),
                        file_id: file.file_id.clone(),
                        absolute_path: file.absolute_path.clone(),
                        line_start: line_no + 1,
                        line_end: line_no + 1,
                        byte_start: hit.byte_start,
                        byte_end: hit.byte_end,
                        value: None,
                        scope: None,
                        owner_component: file.logical_owner.clone(),
                        target_profile: None,
                        confidence: SymbolConfidence::Parsed,
                        mutable_policy: file.mutable_policy,
                    });
                }
                let occurrence_id = format!(
                    "occ:{}:{}:{}:{}",
                    file.file_id,
                    line_no + 1,
                    hit.byte_start,
                    normalized
                );
                self.occurrences.push(DbOccurrenceRow {
                    occurrence_id,
                    symbol_id: symbol_id.clone(),
                    file_id: file.file_id.clone(),
                    match_text: hit.raw.clone(),
                    normalized_text: normalized,
                    line: line_no + 1,
                    column: hit.column + 1,
                    byte_start: hit.byte_start,
                    byte_end: hit.byte_end,
                    context_before: line[..hit.column].to_string(),
                    context_after: line[hit.byte_end_in_line..].to_string(),
                    replace_candidate: replace_policy == ReplacePolicy::Safe
                        || replace_policy == ReplacePolicy::NeedsParser
                        || replace_policy == ReplacePolicy::NeedsOwnerMarker,
                    replace_policy,
                });
            }
        }
    }
}

/// One environment/path-token reference found on a line.
struct EnvRef {
    /// The variable name without `$`/`{}` (e.g. `META_ROOT`).
    name: String,
    /// The raw matched text (e.g. `${META_ROOT}`).
    raw: String,
    /// Byte offset of the match start within the whole-file line stream is not
    /// tracked here; `byte_start` is the column offset within the line, which is
    /// what the refactor previewer needs for in-line replacement.
    byte_start: usize,
    byte_end: usize,
    column: usize,
    byte_end_in_line: usize,
}

/// Find `$VAR`, `${VAR}` references on a single line. Bare-word roots are left
/// to the refactor planner's token-form resolution to avoid false positives on
/// ordinary identifiers.
fn scan_line_env_refs(line: &str) -> Vec<EnvRef> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let (name, raw_len) = if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                // ${VAR}
                let start = i + 2;
                let mut j = start;
                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }
                if j < bytes.len() && is_var_name(&line[start..j]) {
                    (line[start..j].to_string(), (j + 1) - i)
                } else {
                    i += 1;
                    continue;
                }
            } else {
                // $VAR
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && is_var_char(bytes[j]) {
                    j += 1;
                }
                if j > start && is_var_name(&line[start..j]) {
                    (line[start..j].to_string(), j - i)
                } else {
                    i += 1;
                    continue;
                }
            };
            out.push(EnvRef {
                name,
                raw: line[i..i + raw_len].to_string(),
                byte_start: i,
                byte_end: i + raw_len,
                column: i,
                byte_end_in_line: i + raw_len,
            });
            i += raw_len;
        } else {
            i += 1;
        }
    }
    out
}

fn is_var_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// A plausible env-var name: UPPER_SNAKE with at least one letter.
fn is_var_name(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        && s.bytes().any(|b| b.is_ascii_uppercase())
}

fn classify_symbol(normalized: &str, _file_kind: &str) -> DbSymbolKind {
    if normalized.ends_with("_ROOT")
        || normalized.ends_with("_HOME")
        || normalized.ends_with("_DIR")
    {
        DbSymbolKind::PathToken
    } else {
        DbSymbolKind::EnvVar
    }
}

fn kind_tag(kind: &DbSymbolKind) -> &'static str {
    match kind {
        DbSymbolKind::PathToken => "path",
        DbSymbolKind::EnvVar => "env",
        _ => "other",
    }
}

/// Map a file's mutable policy to how safely an occurrence in it can be rewritten.
fn replace_policy_for(policy: MutablePolicy) -> ReplacePolicy {
    match policy {
        MutablePolicy::Never => ReplacePolicy::Refuse,
        MutablePolicy::ReadOnly => ReplacePolicy::ManualReview,
        MutablePolicy::RenderOnly => ReplacePolicy::NeedsOwnerMarker,
        MutablePolicy::OwnedApply => ReplacePolicy::Safe,
        MutablePolicy::GuardedApply => ReplacePolicy::NeedsParser,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_index::{FileIndex, ScanScope};
    use std::fs;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-symbols-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn extracts_root_tokens_with_alias_normalization_and_safe_policy() {
        let root = tmp();
        // shell wrapper (OwnedApply -> Safe replace) referencing both spellings.
        fs::write(
            root.join("wrapper.sh"),
            b"cd \"$META_ROOT/usr/bin\"\nexport OUT=${LIFEOS_ROOT}/opt\n",
        )
        .unwrap();
        // protected file (Never -> Refuse).
        fs::write(root.join(".env"), b"SECRET_DIR=$META_ROOT/secrets\n").unwrap();

        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();

        // META_ROOT and LIFE_OS_ROOT (normalized from LIFEOS_ROOT) are symbols.
        let names: Vec<_> = symbols
            .symbols()
            .iter()
            .map(|s| s.normalized_name.as_str())
            .collect();
        assert!(names.contains(&"META_ROOT"), "got {names:?}");
        assert!(
            names.contains(&"LIFE_OS_ROOT"),
            "LIFEOS_ROOT must normalize"
        );

        // The ${LIFEOS_ROOT} occurrence normalizes but preserves raw match text.
        let lifeos_occ = symbols
            .occurrences()
            .iter()
            .find(|o| o.normalized_text == "LIFE_OS_ROOT")
            .expect("lifeos occurrence");
        assert_eq!(lifeos_occ.match_text, "${LIFEOS_ROOT}");
        assert!(lifeos_occ.replace_candidate); // in a .sh (OwnedApply -> Safe)

        // Occurrence inside .env is Refuse and not a replace candidate.
        let env_occ = symbols
            .occurrences()
            .iter()
            .find(|o| o.file_id.starts_with("file:") && o.context_before.contains("SECRET_DIR"))
            .expect("env occurrence");
        assert_eq!(env_occ.replace_policy, ReplacePolicy::Refuse);
        assert!(!env_occ.replace_candidate);

        let _ = fs::remove_dir_all(&root);
    }
}
