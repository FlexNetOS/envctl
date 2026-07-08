//! db_index — scalable file index over repo/control-plane files (REQ-052).
//!
//! The walk is a bounded, deterministic `std::fs` recursion (no new dependency,
//! keeping the no-C trust boundary trivially intact — REQ-060). It skips heavy
//! generated trees (`.git`, `target`, `node_modules`, …) by default and can be
//! pointed at a narrow subdir via [`ScanScope::root`]. Full `.gitignore`
//! semantics via the `ignore` crate remain an opt-in dependency decision gated
//! by REQ-060. Content is hashed with `sha2` (already an engine dep).

use crate::db::{DbError, MutablePolicy, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Directory names never descended into by default — heavy, generated, or VCS
/// internals. A narrow [`ScanScope::root`] still overrides breadth.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".gitnexus",
    ".venv",
    "__pycache__",
    ".mypy_cache",
];

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

    /// Build the index from a scope: a bounded, deterministic recursion under
    /// `scope.root` (+ any `extra_roots`), skipping [`SKIP_DIRS`], hashing each
    /// file's content and classifying its mutable policy. Rows are sorted by
    /// absolute path so output is deterministic (a repo standard).
    pub fn scan(scope: &ScanScope) -> Result<Self> {
        let mut files = Vec::new();
        let root = Path::new(&scope.root);
        walk(root, root, &mut files)?;
        for extra in &scope.extra_roots {
            let er = Path::new(extra);
            walk(er, er, &mut files)?;
        }
        files.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));
        files.dedup_by(|a, b| a.absolute_path == b.absolute_path);
        Ok(Self { files })
    }
}

/// Recurse `dir`, appending a [`DbFileRow`] per regular file. `base` is the
/// scan root used to derive repo-relative paths.
fn walk(base: &Path, dir: &Path, out: &mut Vec<DbFileRow>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let entries =
        std::fs::read_dir(dir).map_err(|e| DbError::Index(format!("{}: {e}", dir.display())))?;
    let mut children: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    children.sort();
    for path in children {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_symlink() {
            continue; // never follow symlinks (avoid cycles / escaping the scope)
        }
        if path.is_dir() {
            if SKIP_DIRS.contains(&name) {
                continue;
            }
            walk(base, &path, out)?;
        } else if path.is_file() {
            if let Some(row) = index_file(base, &path)? {
                out.push(row);
            }
        }
    }
    Ok(())
}

/// Build one [`DbFileRow`] for a regular file. Returns `Ok(None)` for files that
/// cannot be read as bytes (e.g. permission denied) rather than failing the scan.
fn index_file(base: &Path, path: &Path) -> Result<Option<DbFileRow>> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let content_hash = format!("{:x}", hasher.finalize());
    let line_count = bytecount_lines(&bytes);
    let abs = path.display().to_string();
    let rel = path
        .strip_prefix(base)
        .ok()
        .map(|p| p.display().to_string());
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let (file_kind, parser_hint) = classify(ext, name);
    let generated = rel.as_deref().is_some_and(is_generated_path);
    let protected = is_protected(name, ext);
    let mutable_policy = policy_for(&file_kind, generated, protected);
    Ok(Some(DbFileRow {
        file_id: format!("file:{content_hash}"),
        absolute_path: abs,
        repo_relative_path: rel,
        logical_owner: None,
        file_kind,
        parser_hint,
        content_hash,
        byte_len: bytes.len() as u64,
        line_count,
        generated,
        protected,
        mutable_policy,
        // Deterministic sentinel: REQ-057 (watch) stamps real times; keeping this
        // stable keeps scan output reproducible for content-hash comparison.
        last_indexed_at: String::new(),
    }))
}

fn bytecount_lines(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let nl = bytes.iter().filter(|&&b| b == b'\n').count();
    // Count a final unterminated line.
    if bytes.last() == Some(&b'\n') {
        nl
    } else {
        nl + 1
    }
}

/// (file_kind, parser_hint) from extension / filename.
fn classify(ext: &str, name: &str) -> (String, String) {
    match ext {
        "rs" => ("rust".into(), "syn".into()),
        "toml" => ("toml".into(), "toml_edit".into()),
        "yaml" | "yml" => ("yaml".into(), "serde_yaml".into()),
        "json" => ("json".into(), "serde_json".into()),
        "nu" => ("nushell".into(), "line".into()),
        "sh" | "bash" => ("shell".into(), "line".into()),
        "md" => ("markdown".into(), "text".into()),
        _ => {
            if name.starts_with('.') || name.contains("rc") {
                ("config".into(), "line".into())
            } else {
                ("other".into(), "bytes".into())
            }
        }
    }
}

fn is_generated_path(rel: &str) -> bool {
    rel.contains("/generated/")
        || rel.starts_with("generated/")
        || rel.ends_with(".lock")
        || rel.contains("/dist/")
}

fn is_protected(name: &str, ext: &str) -> bool {
    matches!(ext, "pem" | "key")
        || name == ".env"
        || name.starts_with(".env.")
        || name.ends_with("_secret")
}

fn policy_for(file_kind: &str, generated: bool, protected: bool) -> MutablePolicy {
    if protected {
        return MutablePolicy::Never;
    }
    if generated {
        return MutablePolicy::RenderOnly;
    }
    match file_kind {
        // Structured files we can rewrite with a real parser + owner markers.
        "rust" | "toml" | "yaml" | "json" => MutablePolicy::GuardedApply,
        "shell" | "nushell" | "config" => MutablePolicy::OwnedApply,
        _ => MutablePolicy::ReadOnly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-index-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn scan_indexes_hashes_classifies_and_skips_heavy_trees() {
        let root = tmp();
        fs::write(root.join("main.rs"), b"fn main() {}\n").unwrap();
        fs::write(root.join(".env"), b"SECRET=x\n").unwrap();
        fs::create_dir_all(root.join("generated")).unwrap();
        fs::write(root.join("generated/out.json"), b"{}\n").unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n").unwrap();

        let idx = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();

        let by_rel = |rel: &str| {
            idx.files()
                .iter()
                .find(|f| f.repo_relative_path.as_deref() == Some(rel))
        };

        // .git content is skipped entirely.
        assert!(by_rel(".git/HEAD").is_none(), ".git must be skipped");

        // Rust file: classified + guarded-apply + hashed + line-counted.
        let main = by_rel("main.rs").expect("main.rs indexed");
        assert_eq!(main.file_kind, "rust");
        assert_eq!(main.parser_hint, "syn");
        assert_eq!(main.mutable_policy, MutablePolicy::GuardedApply);
        assert_eq!(main.line_count, 1);
        assert_eq!(main.byte_len, 13); // "fn main() {}\n"
        assert_eq!(main.content_hash.len(), 64); // sha256 hex

        // Protected: .env is Never.
        assert_eq!(by_rel(".env").unwrap().mutable_policy, MutablePolicy::Never);

        // Generated path: render-only.
        let gen = by_rel("generated/out.json").expect("generated file indexed");
        assert!(gen.generated);
        assert_eq!(gen.mutable_policy, MutablePolicy::RenderOnly);

        // Deterministic order.
        let paths: Vec<_> = idx.files().iter().map(|f| &f.absolute_path).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_of_missing_root_is_empty_not_error() {
        let idx = FileIndex::scan(&ScanScope {
            root: "/no/such/path/envctl-xyz".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(idx.files().is_empty());
    }
}
