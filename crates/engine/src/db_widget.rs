//! db_widget — compact agent-UI surfaces over the db snapshot (REQ CMD08/09/10,
//! ARCH11, AC10).
//!
//! These are the read-only, machine-shaped views an agent front-end renders
//! directly: [`roots_widget`] (the multi-root model plus per-root reference
//! counts), [`refs_widget`] (every env-var / path-token reference grouped by
//! symbol), and [`hooks_widget`] (discovered hook/wrapper scripts with their
//! mutable policy). All logic lives here in the engine so the CLI and GUI render
//! the identical bytes.

use crate::db::EnvRootRow;
use crate::db::MutablePolicy;
use crate::db_index::FileIndex;
use crate::db_symbols::{DbSymbolKind, ReplacePolicy, SymbolIndex};
use serde::{Deserialize, Serialize};

/// Per-root reference tally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootRefCount {
    pub normalized_name: String,
    pub occurrences: usize,
    pub files: usize,
}

/// `db widget roots` — the multi-root model + how often each root is referenced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootsWidget {
    pub roots: Vec<EnvRootRow>,
    pub reference_counts: Vec<RootRefCount>,
}

/// One reference location under a symbol group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefLocation {
    pub absolute_path: String,
    pub repo_relative_path: Option<String>,
    pub line: usize,
    pub column: usize,
    pub match_text: String,
    pub replace_policy: ReplacePolicy,
    pub replace_candidate: bool,
}

/// A symbol and all its references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefGroup {
    pub symbol_id: String,
    pub normalized_name: String,
    pub kind: DbSymbolKind,
    pub occurrences: Vec<RefLocation>,
}

/// `db widget refs` — every reference grouped by symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefsWidget {
    pub symbols: Vec<RefGroup>,
}

/// A discovered hook/wrapper script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEntry {
    pub absolute_path: String,
    pub repo_relative_path: Option<String>,
    pub file_kind: String,
    pub mutable_policy: MutablePolicy,
    pub protected: bool,
    pub generated: bool,
}

/// `db widget hooks` — hook/wrapper scripts with their mutable policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HooksWidget {
    pub hooks: Vec<HookEntry>,
}

/// Build the roots widget: the passed multi-root model plus per-root reference
/// counts derived from the symbol index (deterministic order).
pub fn roots_widget(roots: Vec<EnvRootRow>, symbols: &SymbolIndex) -> RootsWidget {
    use std::collections::BTreeMap;
    let mut occ: BTreeMap<&str, usize> = BTreeMap::new();
    let mut files: BTreeMap<&str, std::collections::BTreeSet<&str>> = BTreeMap::new();
    for o in symbols.occurrences() {
        *occ.entry(o.normalized_text.as_str()).or_insert(0) += 1;
        files
            .entry(o.normalized_text.as_str())
            .or_default()
            .insert(o.file_id.as_str());
    }
    let reference_counts = occ
        .into_iter()
        .map(|(name, occurrences)| RootRefCount {
            normalized_name: name.to_string(),
            occurrences,
            files: files.get(name).map(|s| s.len()).unwrap_or(0),
        })
        .collect();
    RootsWidget {
        roots,
        reference_counts,
    }
}

/// Build the refs widget: occurrences grouped under their symbol (deterministic).
/// `files` resolves each occurrence's `file_id` to its real path (an occurrence
/// can live in a file other than where its symbol was first registered).
pub fn refs_widget(files: &FileIndex, symbols: &SymbolIndex) -> RefsWidget {
    use std::collections::BTreeMap;
    let by_id: BTreeMap<&str, (&str, Option<&str>)> = files
        .files()
        .iter()
        .map(|f| {
            (
                f.file_id.as_str(),
                (f.absolute_path.as_str(), f.repo_relative_path.as_deref()),
            )
        })
        .collect();
    let mut groups = Vec::new();
    for sym in symbols.symbols() {
        let occurrences = symbols
            .occurrences()
            .iter()
            .filter(|o| o.symbol_id == sym.symbol_id)
            .map(|o| {
                let (abs, rel) = by_id
                    .get(o.file_id.as_str())
                    .map(|(a, r)| (a.to_string(), r.map(str::to_string)))
                    .unwrap_or_else(|| (sym.absolute_path.clone(), None));
                RefLocation {
                    absolute_path: abs,
                    repo_relative_path: rel,
                    line: o.line,
                    column: o.column,
                    match_text: o.match_text.clone(),
                    replace_policy: o.replace_policy,
                    replace_candidate: o.replace_candidate,
                }
            })
            .collect();
        groups.push(RefGroup {
            symbol_id: sym.symbol_id.clone(),
            normalized_name: sym.normalized_name.clone(),
            kind: sym.kind.clone(),
            occurrences,
        });
    }
    groups.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));
    RefsWidget { symbols: groups }
}

/// Build the hooks widget: shell/nushell scripts (hook + wrapper surfaces) with
/// their mutable policy, so an agent can see what is safe to deploy/rewrite.
pub fn hooks_widget(files: &FileIndex) -> HooksWidget {
    let mut hooks: Vec<HookEntry> = files
        .files()
        .iter()
        .filter(|f| matches!(f.file_kind.as_str(), "shell" | "nushell"))
        .map(|f| HookEntry {
            absolute_path: f.absolute_path.clone(),
            repo_relative_path: f.repo_relative_path.clone(),
            file_kind: f.file_kind.clone(),
            mutable_policy: f.mutable_policy,
            protected: f.protected,
            generated: f.generated,
        })
        .collect();
    hooks.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));
    HooksWidget { hooks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_index::ScanScope;
    use crate::db_ops::roots;
    use std::fs;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-widget-{}", std::process::id()));
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

    #[test]
    fn roots_refs_and_hooks_widgets_are_populated_and_deterministic() {
        let root = tmp();
        fs::write(root.join("w.sh"), b"cd $META_ROOT/bin\nX=${META_ROOT}/x\n").unwrap();
        fs::write(root.join("hook.nu"), b"cd $LIFE_OS_ROOT\n").unwrap();
        fs::write(root.join(".env"), b"S=$META_ROOT/s\n").unwrap();
        let (files, symbols) = indexes(&root);

        // roots widget: the multi-root model plus reference tallies.
        let rw = roots_widget(
            roots(Some("/o".into()), Some("/r".into()), "lifeos-release"),
            &symbols,
        );
        assert_eq!(rw.roots.len(), 2);
        let meta = rw
            .reference_counts
            .iter()
            .find(|c| c.normalized_name == "META_ROOT")
            .expect("META_ROOT counted");
        assert_eq!(meta.occurrences, 3, "2 in w.sh + 1 in .env");
        assert_eq!(meta.files, 2);
        // Deterministic: re-run yields identical JSON.
        let a = serde_json::to_string(&roots_widget(
            roots(Some("/o".into()), Some("/r".into()), "lifeos-release"),
            &symbols,
        ))
        .unwrap();
        let b = serde_json::to_string(&rw).unwrap();
        assert_eq!(a, b);

        // refs widget: grouped by symbol, sorted.
        let refs = refs_widget(&files, &symbols);
        assert!(refs
            .symbols
            .iter()
            .any(|g| g.normalized_name == "META_ROOT" && g.occurrences.len() == 3));

        // hooks widget: the .sh + .nu scripts, not the .env.
        let hooks = hooks_widget(&files);
        assert_eq!(hooks.hooks.len(), 2);
        assert!(hooks.hooks.iter().any(|h| h.file_kind == "shell"));
        assert!(hooks.hooks.iter().any(|h| h.file_kind == "nushell"));
        assert!(!hooks
            .hooks
            .iter()
            .any(|h| h.absolute_path.ends_with(".env")));

        let _ = fs::remove_dir_all(&root);
    }
}
