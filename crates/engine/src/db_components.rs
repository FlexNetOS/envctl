//! db_components — the managed structural-tooling inventory + detection
//! (REQ NFR05 / ARCH10 / AC09).
//!
//! envctl leans on a small set of pure-Rust / trust-boundary-safe CLIs (`rg`,
//! `sg`, `fd`, `sd`, `taplo`, `jaq`, `nu`) for polyglot structural extraction and
//! safe edits. [`detect`] probes each against `PATH` and the nix profile — a
//! read-only, local resolution only. **It never installs anything and never
//! touches the network**; provisioning stays a separate, explicit step.
//!
//! The canonical inventory is [`INVENTORY`] below; `components/managed-tools.toml`
//! is the human-facing mirror, pinned to this const by a parity test.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// One managed tool in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedTool {
    pub id: &'static str,
    pub binary: &'static str,
    pub purpose: &'static str,
    pub category: &'static str,
}

/// The canonical managed-tool inventory (source of truth for detection).
pub const INVENTORY: &[ManagedTool] = &[
    ManagedTool {
        id: "rg",
        binary: "rg",
        purpose: "ripgrep: fast literal/regex content search across the tree",
        category: "search",
    },
    ManagedTool {
        id: "sg",
        binary: "sg",
        purpose: "ast-grep: structural (AST) search + rewrite for polyglot extraction",
        category: "structural",
    },
    ManagedTool {
        id: "fd",
        binary: "fd",
        purpose: "fd: fast, gitignore-aware file discovery",
        category: "search",
    },
    ManagedTool {
        id: "sd",
        binary: "sd",
        purpose: "sd: safe, literal find-and-replace for line-oriented edits",
        category: "edit",
    },
    ManagedTool {
        id: "taplo",
        binary: "taplo",
        purpose: "taplo: format-preserving TOML parse/query/edit",
        category: "format",
    },
    ManagedTool {
        id: "jaq",
        binary: "jaq",
        purpose: "jaq: pure-Rust jq for JSON query/transform",
        category: "format",
    },
    ManagedTool {
        id: "nu",
        binary: "nu",
        purpose: "nushell: structured-data shell driving envctl's gates and pipelines",
        category: "shell",
    },
];

/// The detection result for one tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDetection {
    pub id: String,
    pub binary: String,
    pub purpose: String,
    pub category: String,
    pub present: bool,
    /// Absolute path the binary resolved to, if found.
    pub resolved_path: Option<String>,
    /// Where it was found: `"path"` or `"nix-profile"`.
    pub source: Option<String>,
}

/// The full inventory-detection report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentsReport {
    pub tools: Vec<ToolDetection>,
    pub present: usize,
    pub missing: usize,
}

/// True when `p` is a regular file with an executable bit set (unix). On other
/// platforms, existence as a file is sufficient.
fn is_executable_file(p: &Path) -> bool {
    let meta = match std::fs::metadata(p) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Resolve `binary` against an ordered list of `(dir, source-label)`. First
/// executable hit wins (so PATH shadows the nix profile, matching shell lookup).
fn resolve_in(binary: &str, dirs: &[(PathBuf, &'static str)]) -> Option<(PathBuf, &'static str)> {
    for (dir, source) in dirs {
        let candidate = dir.join(binary);
        if is_executable_file(&candidate) {
            return Some((candidate, source));
        }
    }
    None
}

/// The `PATH` directories, in order.
fn path_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default()
}

/// Well-known nix profile `bin` directories (checked after `PATH`). Includes the
/// per-user profile, the system default profile, and `$NIX_PROFILE/bin` when set.
fn nix_profile_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(&home).join(".nix-profile/bin"));
    }
    if let Some(np) = std::env::var_os("NIX_PROFILE") {
        dirs.push(PathBuf::from(np).join("bin"));
    }
    dirs.push(PathBuf::from("/nix/var/nix/profiles/default/bin"));
    dirs
}

/// Detect the whole inventory against real `PATH` + nix profile dirs.
pub fn detect() -> ComponentsReport {
    let mut search: Vec<(PathBuf, &'static str)> =
        path_dirs().into_iter().map(|d| (d, "path")).collect();
    search.extend(nix_profile_dirs().into_iter().map(|d| (d, "nix-profile")));
    detect_with(&search)
}

/// Detect the inventory against an explicit, ordered search list (testable).
pub fn detect_with(search: &[(PathBuf, &'static str)]) -> ComponentsReport {
    let mut tools = Vec::with_capacity(INVENTORY.len());
    let (mut present, mut missing) = (0usize, 0usize);
    for tool in INVENTORY {
        let hit = resolve_in(tool.binary, search);
        if hit.is_some() {
            present += 1;
        } else {
            missing += 1;
        }
        tools.push(ToolDetection {
            id: tool.id.to_string(),
            binary: tool.binary.to_string(),
            purpose: tool.purpose.to_string(),
            category: tool.category.to_string(),
            present: hit.is_some(),
            resolved_path: hit.as_ref().map(|(p, _)| p.display().to_string()),
            source: hit.as_ref().map(|(_, s)| s.to_string()),
        });
    }
    ComponentsReport {
        tools,
        present,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("envctl-db-components-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[cfg(unix)]
    fn write_exe(dir: &Path, name: &str) {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        fs::write(&p, b"#!/bin/sh\ntrue\n").unwrap();
        let mut perms = fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&p, perms).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn detect_finds_present_tools_and_records_source_and_path() {
        let path_dir = tmp("path");
        let nix_dir = tmp("nix");
        // rg on PATH, nu only in the nix profile; a non-executable `fd` is NOT a hit.
        write_exe(&path_dir, "rg");
        write_exe(&nix_dir, "nu");
        fs::write(path_dir.join("fd"), b"not executable").unwrap();

        let search = vec![(path_dir.clone(), "path"), (nix_dir.clone(), "nix-profile")];
        let report = detect_with(&search);
        assert_eq!(report.tools.len(), INVENTORY.len());

        let by_id = |id: &str| report.tools.iter().find(|t| t.id == id).unwrap();
        let rg = by_id("rg");
        assert!(rg.present);
        assert_eq!(rg.source.as_deref(), Some("path"));
        assert_eq!(
            rg.resolved_path.as_deref(),
            Some(path_dir.join("rg").display().to_string().as_str())
        );

        let nu = by_id("nu");
        assert!(nu.present);
        assert_eq!(nu.source.as_deref(), Some("nix-profile"));

        // Non-executable fd -> not present.
        assert!(!by_id("fd").present);
        // taplo absent entirely.
        assert!(!by_id("taplo").present);

        assert_eq!(report.present, 2, "rg + nu");
        assert_eq!(report.missing, INVENTORY.len() - 2);

        let _ = fs::remove_dir_all(&path_dir);
        let _ = fs::remove_dir_all(&nix_dir);
    }

    #[test]
    fn manifest_matches_canonical_inventory() {
        // The human-facing components/managed-tools.toml must list exactly the
        // canonical INVENTORY ids/binaries (no drift).
        let manifest =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../components/managed-tools.toml");
        let text = fs::read_to_string(&manifest)
            .unwrap_or_else(|e| panic!("read {}: {e}", manifest.display()));
        let doc: toml::Value = toml::from_str(&text).unwrap();
        let tools = doc
            .get("tool")
            .and_then(|t| t.as_array())
            .expect("[[tool]] array");
        assert_eq!(
            tools.len(),
            INVENTORY.len(),
            "manifest/inventory count drift"
        );
        for (row, canonical) in tools.iter().zip(INVENTORY.iter()) {
            assert_eq!(row.get("id").and_then(|v| v.as_str()), Some(canonical.id));
            assert_eq!(
                row.get("binary").and_then(|v| v.as_str()),
                Some(canonical.binary)
            );
            assert_eq!(
                row.get("category").and_then(|v| v.as_str()),
                Some(canonical.category)
            );
        }
    }
}
