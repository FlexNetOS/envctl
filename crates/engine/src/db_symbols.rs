//! db_symbols — symbol + occurrence index over indexed files (REQ-053).
//!
//! REQ-050 scaffold: the [`DbSymbolKind`], [`DbSymbolRow`], and
//! [`DbOccurrenceRow`] shapes plus the [`SymbolIndex`] seam. Rust symbols come
//! from `syn` in-core; polyglot structural matching (ast-grep/tree-sitter) is
//! wired as an external managed component so the no-C gate holds (REQ-053/060).

use crate::db::{normalize_root_var, MutablePolicy, Result};
use crate::db_index::{DbFileRow, FileIndex};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
            // Structural Rust-item extraction (crate/module/item/import/clap) via
            // `syn` — pure Rust, no C in the trust boundary (ARCH09). A file that
            // fails to parse (partial/edition-mismatched) is skipped, never fatal.
            if file.file_kind == "rust" {
                idx.scan_rust_items(file, &content);
            }
            if matches!(file.file_kind.as_str(), "shell" | "nushell") {
                idx.scan_script_symbols(file);
            }
            if matches!(file.file_kind.as_str(), "config" | "toml" | "yaml" | "json") {
                idx.scan_config_key_symbols(file, &content);
            }
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
        let mut line_byte_start = 0usize;
        for (line_no, line) in content.lines().enumerate() {
            for hit in scan_line_env_refs(line) {
                let normalized = normalize_root_var(&hit.name);
                let kind = classify_symbol(&normalized, &file.file_kind);
                let symbol_id = format!("sym:{}:{}", kind_tag(&kind), normalized);
                let byte_start = line_byte_start + hit.byte_start;
                let byte_end = line_byte_start + hit.byte_end;
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
                        byte_start,
                        byte_end,
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
                    byte_start,
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
                    byte_start,
                    byte_end,
                    context_before: line[..hit.column].to_string(),
                    context_after: line[hit.byte_end_in_line..].to_string(),
                    replace_candidate: replace_policy == ReplacePolicy::Safe
                        || replace_policy == ReplacePolicy::NeedsParser
                        || replace_policy == ReplacePolicy::NeedsOwnerMarker,
                    replace_policy,
                });
            }
            // `str::lines` strips both LF and CRLF terminators. Advancing by
            // the content byte immediately following this line preserves
            // whole-file byte spans for either form and for a final line with
            // no terminator.
            line_byte_start += line.len();
            if content.as_bytes().get(line_byte_start) == Some(&b'\r') {
                line_byte_start += 1;
            }
            if content.as_bytes().get(line_byte_start) == Some(&b'\n') {
                line_byte_start += 1;
            }
        }
    }

    /// Scan one shell/nushell script as a hook/wrapper symbol.
    ///
    /// Hook/wrapper symbols are discovered from path heuristics to keep detection
    /// fast and deterministic; each yields one occurrence so `db symbols --kind
    /// hook-script` and `--kind wrapper-script` remain usable via symbol/occurrence
    /// joins.
    fn scan_script_symbols(&mut self, file: &DbFileRow) {
        let replace_policy = replace_policy_for(file.mutable_policy);
        let kind = classify_script_kind(file);
        let symbol_name = file
            .repo_relative_path
            .as_deref()
            .unwrap_or(file.absolute_path.as_str())
            .to_owned();
        let display_name = Path::new(&symbol_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&symbol_name);
        let symbol_id = format!("sym:{}:{}:{}", kind_tag(&kind), file.file_id, symbol_name);

        if !self.symbols.iter().any(|s| s.symbol_id == symbol_id) {
            self.symbols.push(DbSymbolRow {
                symbol_id: symbol_id.clone(),
                kind,
                name: display_name.to_string(),
                normalized_name: symbol_name.clone(),
                file_id: file.file_id.clone(),
                absolute_path: file.absolute_path.clone(),
                line_start: 1,
                line_end: 1,
                byte_start: 0,
                byte_end: 0,
                value: None,
                scope: None,
                owner_component: file.logical_owner.clone(),
                target_profile: None,
                confidence: SymbolConfidence::Parsed,
                mutable_policy: file.mutable_policy,
            });
        }

        self.occurrences.push(DbOccurrenceRow {
            occurrence_id: format!("occ:{}:{}:{}:{}", file.file_id, 1, 0, symbol_name),
            symbol_id,
            file_id: file.file_id.clone(),
            match_text: String::new(),
            normalized_text: symbol_name,
            line: 1,
            column: 1,
            byte_start: 0,
            byte_end: 0,
            context_before: String::new(),
            context_after: String::new(),
            replace_candidate: replace_policy == ReplacePolicy::Safe
                || replace_policy == ReplacePolicy::NeedsParser
                || replace_policy == ReplacePolicy::NeedsOwnerMarker,
            replace_policy,
        });
    }

    /// Scan config-like files (toml/yaml/json/config) for symbol-key occurrences.
    ///
    /// This pass is intentionally line-local and heuristic; it is a low-cost symbol
    /// discovery pass, not a schema validator.
    fn scan_config_key_symbols(&mut self, file: &DbFileRow, content: &str) {
        let replace_policy = replace_policy_for(file.mutable_policy);
        let mut line_byte_start = 0usize;
        for (line_no, line) in content.lines().enumerate() {
            for hit in scan_config_keys_in_line(file.file_kind.as_str(), line) {
                let symbol_id = format!(
                    "sym:{}:{}:{}",
                    kind_tag(&DbSymbolKind::ConfigKey),
                    file.file_id,
                    hit.name
                );
                if !self.symbols.iter().any(|s| s.symbol_id == symbol_id) {
                    self.symbols.push(DbSymbolRow {
                        symbol_id: symbol_id.clone(),
                        kind: DbSymbolKind::ConfigKey,
                        name: hit.name.clone(),
                        normalized_name: hit.name.clone(),
                        file_id: file.file_id.clone(),
                        absolute_path: file.absolute_path.clone(),
                        line_start: line_no + 1,
                        line_end: line_no + 1,
                        byte_start: line_byte_start + hit.byte_start,
                        byte_end: line_byte_start + hit.byte_end,
                        value: None,
                        scope: None,
                        owner_component: file.logical_owner.clone(),
                        target_profile: None,
                        confidence: SymbolConfidence::Parsed,
                        mutable_policy: file.mutable_policy,
                    });
                }
                self.occurrences.push(DbOccurrenceRow {
                    occurrence_id: format!(
                        "occ:{}:{}:{}:{}",
                        file.file_id,
                        line_no + 1,
                        line_byte_start + hit.byte_start,
                        hit.name
                    ),
                    symbol_id: symbol_id.clone(),
                    file_id: file.file_id.clone(),
                    match_text: hit.raw.clone(),
                    normalized_text: hit.name.clone(),
                    line: line_no + 1,
                    column: hit.column + 1,
                    byte_start: line_byte_start + hit.byte_start,
                    byte_end: line_byte_start + hit.byte_end,
                    context_before: line[..hit.byte_start].to_string(),
                    context_after: line[hit.byte_end_in_line..].to_string(),
                    replace_candidate: replace_policy == ReplacePolicy::Safe
                        || replace_policy == ReplacePolicy::NeedsParser
                        || replace_policy == ReplacePolicy::NeedsOwnerMarker,
                    replace_policy,
                });
            }
            line_byte_start += line.len();
            if content.as_bytes().get(line_byte_start) == Some(&b'\r') {
                line_byte_start += 1;
            }
            if content.as_bytes().get(line_byte_start) == Some(&b'\n') {
                line_byte_start += 1;
            }
        }
    }

    /// syn-based structural extraction over a Rust source file (ARCH09): emits a
    /// [`DbSymbolRow`] per top-level and nested item — crate/module/fn/struct/enum/
    /// trait/impl/const/static/type, `use` imports, and clap `derive(Parser|
    /// Subcommand)` types (surfaced as [`DbSymbolKind::CliSubcommand`]). Definitions
    /// only (no occurrence rows), so the env-token refactor surface is untouched.
    fn scan_rust_items(&mut self, file: &DbFileRow, content: &str) {
        let ast = match syn::parse_file(content) {
            Ok(a) => a,
            Err(_) => return, // unparseable -> skip, never fail the whole build
        };
        let mut items = Vec::new();
        collect_rust_items(&ast.items, "crate", &mut items);
        for it in items {
            let symbol_id = format!(
                "sym:rust:{}:{}:{}",
                file.file_id, it.line, it.qualified_name
            );
            if self.symbols.iter().any(|s| s.symbol_id == symbol_id) {
                continue;
            }
            self.symbols.push(DbSymbolRow {
                symbol_id,
                kind: it.kind,
                name: it.name,
                normalized_name: it.qualified_name,
                file_id: file.file_id.clone(),
                absolute_path: file.absolute_path.clone(),
                line_start: it.line,
                line_end: it.line,
                byte_start: 0,
                byte_end: 0,
                value: Some(it.item_kind.to_string()),
                scope: Some(it.module_path),
                owner_component: file.logical_owner.clone(),
                target_profile: None,
                confidence: SymbolConfidence::Parsed,
                mutable_policy: file.mutable_policy,
            });
        }
        self.symbols.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));
    }
}

/// One structural item extracted from a Rust file.
struct RustItem {
    /// Bare item name (e.g. `run_db`).
    name: String,
    /// Module-qualified name (e.g. `crate::db::run_db`) — the dedupe key.
    qualified_name: String,
    /// The enclosing module path (e.g. `crate::db`).
    module_path: String,
    /// The item category tag (`fn`, `struct`, `enum`, `use`, …).
    item_kind: &'static str,
    /// The symbol kind (`RustItem`, or `CliSubcommand` for clap derives).
    kind: DbSymbolKind,
    /// 1-based source line (from proc-macro2 span-locations).
    line: usize,
}

/// Recurse `items` under `module_path`, appending a [`RustItem`] per definition.
fn collect_rust_items(items: &[syn::Item], module_path: &str, out: &mut Vec<RustItem>) {
    use syn::spanned::Spanned;
    let line_of = |span: proc_macro2::Span| span.start().line;
    let qualify = |name: &str| format!("{module_path}::{name}");

    for item in items {
        match item {
            syn::Item::Fn(f) => out.push(RustItem {
                name: f.sig.ident.to_string(),
                qualified_name: qualify(&f.sig.ident.to_string()),
                module_path: module_path.to_string(),
                item_kind: "fn",
                kind: DbSymbolKind::RustItem,
                line: line_of(f.sig.ident.span()),
            }),
            syn::Item::Struct(s) => {
                let clap = derive_clap_kind(&s.attrs);
                out.push(RustItem {
                    name: s.ident.to_string(),
                    qualified_name: qualify(&s.ident.to_string()),
                    module_path: module_path.to_string(),
                    item_kind: if clap.is_some() {
                        "clap-struct"
                    } else {
                        "struct"
                    },
                    kind: clap.unwrap_or(DbSymbolKind::RustItem),
                    line: line_of(s.ident.span()),
                });
            }
            syn::Item::Enum(e) => {
                let clap = derive_clap_kind(&e.attrs);
                out.push(RustItem {
                    name: e.ident.to_string(),
                    qualified_name: qualify(&e.ident.to_string()),
                    module_path: module_path.to_string(),
                    item_kind: if clap.is_some() { "clap-enum" } else { "enum" },
                    kind: clap.unwrap_or(DbSymbolKind::RustItem),
                    line: line_of(e.ident.span()),
                });
            }
            syn::Item::Trait(t) => out.push(RustItem {
                name: t.ident.to_string(),
                qualified_name: qualify(&t.ident.to_string()),
                module_path: module_path.to_string(),
                item_kind: "trait",
                kind: DbSymbolKind::RustItem,
                line: line_of(t.ident.span()),
            }),
            syn::Item::Const(c) => out.push(RustItem {
                name: c.ident.to_string(),
                qualified_name: qualify(&c.ident.to_string()),
                module_path: module_path.to_string(),
                item_kind: "const",
                kind: DbSymbolKind::RustItem,
                line: line_of(c.ident.span()),
            }),
            syn::Item::Static(s) => out.push(RustItem {
                name: s.ident.to_string(),
                qualified_name: qualify(&s.ident.to_string()),
                module_path: module_path.to_string(),
                item_kind: "static",
                kind: DbSymbolKind::RustItem,
                line: line_of(s.ident.span()),
            }),
            syn::Item::Type(t) => out.push(RustItem {
                name: t.ident.to_string(),
                qualified_name: qualify(&t.ident.to_string()),
                module_path: module_path.to_string(),
                item_kind: "type",
                kind: DbSymbolKind::RustItem,
                line: line_of(t.ident.span()),
            }),
            syn::Item::Use(u) => {
                for path in flatten_use_tree(&u.tree, String::new()) {
                    out.push(RustItem {
                        name: path.rsplit("::").next().unwrap_or(&path).to_string(),
                        qualified_name: qualify(&format!("use {path}")),
                        module_path: module_path.to_string(),
                        item_kind: "use",
                        kind: DbSymbolKind::RustItem,
                        line: line_of(u.span()),
                    });
                }
            }
            syn::Item::Mod(m) => {
                let child = qualify(&m.ident.to_string());
                out.push(RustItem {
                    name: m.ident.to_string(),
                    qualified_name: child.clone(),
                    module_path: module_path.to_string(),
                    item_kind: "mod",
                    kind: DbSymbolKind::RustItem,
                    line: line_of(m.ident.span()),
                });
                if let Some((_, inner)) = &m.content {
                    collect_rust_items(inner, &child, out);
                }
            }
            _ => {}
        }
    }
}

/// If `attrs` carry `#[derive(Parser)]` / `#[derive(Subcommand)]`, return the
/// matching clap symbol kind. `Subcommand` -> [`DbSymbolKind::CliSubcommand`];
/// `Parser`/`Args` -> also `CliSubcommand` (both are the CLI-surface contract).
fn derive_clap_kind(attrs: &[syn::Attribute]) -> Option<DbSymbolKind> {
    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let mut found = false;
        // parse_nested_meta walks the derive list: `#[derive(A, B, C)]`.
        let _ = attr.parse_nested_meta(|meta| {
            if let Some(id) = meta.path.get_ident() {
                let n = id.to_string();
                if n == "Parser" || n == "Subcommand" || n == "Args" {
                    found = true;
                }
            }
            Ok(())
        });
        if found {
            return Some(DbSymbolKind::CliSubcommand);
        }
    }
    None
}

/// Flatten a `use` tree into fully-qualified path strings (one per leaf/glob).
fn flatten_use_tree(tree: &syn::UseTree, prefix: String) -> Vec<String> {
    let join = |p: &str, seg: &str| {
        if p.is_empty() {
            seg.to_string()
        } else {
            format!("{p}::{seg}")
        }
    };
    match tree {
        syn::UseTree::Path(p) => flatten_use_tree(&p.tree, join(&prefix, &p.ident.to_string())),
        syn::UseTree::Name(n) => vec![join(&prefix, &n.ident.to_string())],
        syn::UseTree::Rename(r) => vec![join(&prefix, &r.ident.to_string())],
        syn::UseTree::Glob(_) => vec![join(&prefix, "*")],
        syn::UseTree::Group(g) => g
            .items
            .iter()
            .flat_map(|t| flatten_use_tree(t, prefix.clone()))
            .collect(),
    }
}

fn classify_script_kind(file: &DbFileRow) -> DbSymbolKind {
    let path = file
        .repo_relative_path
        .as_deref()
        .unwrap_or(file.absolute_path.as_str())
        .to_ascii_lowercase();
    let name = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if path.contains("/hooks/")
        || path.ends_with("/hooks")
        || path.contains(".codex/hooks")
        || name.contains("hook")
        || name.ends_with("-hook")
        || name.ends_with(".hook")
        || name.ends_with("git-hook")
    {
        DbSymbolKind::HookScript
    } else if path.contains("/wrappers/")
        || path.contains("wrapper")
        || name == "wrapper.sh"
        || path.contains("/usr/bin/")
    {
        DbSymbolKind::WrapperScript
    } else {
        DbSymbolKind::HookScript
    }
}

/// One config key reference extracted from one line.
struct ConfigRef {
    name: String,
    raw: String,
    byte_start: usize,
    byte_end: usize,
    column: usize,
    byte_end_in_line: usize,
}

/// Best-effort config key extraction by format.
fn scan_config_keys_in_line(file_kind: &str, line: &str) -> Vec<ConfigRef> {
    match file_kind {
        "toml" => scan_toml_config_keys(line),
        "yaml" => scan_yaml_config_keys(line),
        "json" => scan_json_config_keys(line),
        _ => scan_config_style_keys(line),
    }
}

fn is_config_key_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.'
}

fn is_config_key(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(is_config_key_char)
}

fn unquoted_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        Some(trimmed[1..trimmed.len() - 1].to_string())
    } else {
        Some(trimmed.to_string())
    }
}

fn scan_toml_config_keys(line: &str) -> Vec<ConfigRef> {
    let mut out = Vec::new();
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || !trimmed.contains('=') {
        return out;
    }
    if trimmed.starts_with('[') {
        return out;
    }
    let bytes = line.as_bytes();
    let mut eq = 0usize;
    while eq < bytes.len() && bytes[eq] != b'=' {
        eq += 1;
    }
    if eq == 0 || eq >= bytes.len() {
        return out;
    }
    let mut key_start = 0usize;
    while key_start < eq && (bytes[key_start] == b' ' || bytes[key_start] == b'\t') {
        key_start += 1;
    }
    if key_start >= eq {
        return out;
    }
    let raw = line[key_start..eq].trim();
    let key_len = raw.len();
    let key = unquoted_key(raw).unwrap_or_else(|| raw.to_string());
    if !is_config_key(&key) {
        return out;
    }
    out.push(ConfigRef {
        name: key.clone(),
        raw: key,
        byte_start: key_start,
        byte_end: key_start + key_len,
        column: key_start,
        byte_end_in_line: key_start + key_len,
    });
    out
}

fn scan_yaml_config_keys(line: &str) -> Vec<ConfigRef> {
    let mut out = Vec::new();
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || !trimmed.contains(':') {
        return out;
    }
    let mut colon = 0usize;
    while colon < trimmed.len() && trimmed.as_bytes()[colon] != b':' {
        colon += 1;
    }
    if colon == 0 || colon >= trimmed.len() {
        return out;
    }
    let key = trimmed[..colon].trim();
    if key.starts_with('-') || key.is_empty() {
        return out;
    }
    let key = unquoted_key(key).unwrap_or_else(|| key.to_string());
    if !is_config_key(&key) {
        return out;
    }
    let offset = line.len() - trimmed.len();
    out.push(ConfigRef {
        name: key.clone(),
        raw: key.clone(),
        byte_start: offset,
        byte_end: offset + key.len(),
        column: offset,
        byte_end_in_line: offset + key.len(),
    });
    out
}

fn scan_json_config_keys(line: &str) -> Vec<ConfigRef> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }
        let key_start = i + 1;
        let mut j = key_start;
        while j < bytes.len() && bytes[j] != b'"' {
            if bytes[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }
        let mut k = j + 1;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k < bytes.len() && bytes[k] == b':' {
            let key = &line[key_start..j];
            if is_config_key(key) {
                out.push(ConfigRef {
                    name: key.to_string(),
                    raw: key.to_string(),
                    byte_start: key_start,
                    byte_end: j,
                    column: key_start,
                    byte_end_in_line: key_start + key.len(),
                });
            }
        }
        i = j + 1;
    }
    out
}

fn scan_config_style_keys(line: &str) -> Vec<ConfigRef> {
    let mut out = Vec::new();
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return out;
    }
    let text = trimmed.strip_prefix("export ").unwrap_or(trimmed);
    let eq = match text.find('=') {
        Some(i) => i,
        None => return out,
    };
    let key = text[..eq].trim();
    if key.is_empty() || !is_config_key(key) {
        return out;
    }
    if let Some(byte_start) = line.find(key) {
        out.push(ConfigRef {
            name: key.to_string(),
            raw: key.to_string(),
            byte_start,
            byte_end: byte_start + key.len(),
            column: byte_start,
            byte_end_in_line: byte_start + key.len(),
        });
    }
    out
}

/// One environment/path-token reference found on a line.
struct EnvRef {
    /// The variable name without `$`/`{}` (e.g. `META_ROOT`).
    name: String,
    /// The raw matched text (e.g. `${META_ROOT}`).
    raw: String,
    /// Byte offsets within the current line. [`SymbolIndex::scan_file`] adds the
    /// line's whole-file byte offset before storing an occurrence.
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

/// A plausible env-var name: UPPER_SNAKE with at least one letter. Shared with
/// the refactor planner (REQ-055) so rewrite token-boundaries match the scanner.
pub(crate) fn is_var_name(s: &str) -> bool {
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
        DbSymbolKind::HookScript => "hook_script",
        DbSymbolKind::WrapperScript => "wrapper_script",
        DbSymbolKind::ConfigKey => "config_key",
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

/// Per-file slice of a symbol's blast radius.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactFile {
    pub file_id: String,
    pub absolute_path: String,
    pub repo_relative_path: Option<String>,
    pub mutable_policy: MutablePolicy,
    pub occurrence_count: usize,
    /// Occurrences that are safe to mechanically rewrite (replace candidates).
    pub safe: usize,
    /// Occurrences refused (protected/.env/manual-review policy).
    pub refused: usize,
}

/// The read-only impact map for one symbol: every file + occurrence that
/// references it, split by rewrite safety. This is what an agent consults before
/// proposing a refactor (`envctl db impact --symbol <name>`), REQ CMD05.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactReport {
    /// The symbol as requested.
    pub symbol: String,
    /// Its normalized (alias-collapsed) form — the key occurrences match on.
    pub normalized_symbol: String,
    pub files: Vec<ImpactFile>,
    pub files_affected: usize,
    pub occurrences_total: usize,
    pub safe_occurrences: usize,
    pub refused_occurrences: usize,
    /// Symbol rows whose normalized name matches (the "definitions" side).
    pub definitions: Vec<DbSymbolRow>,
}

/// Map the blast radius of `symbol` across the indexed scope. Normalization-aware
/// (the `LIFEOS_ROOT` alias resolves to the same key as `LIFE_OS_ROOT`), and
/// deterministic (files sorted by absolute path). Never mutates anything.
pub fn impact(symbol: &str, files: &FileIndex, symbols: &SymbolIndex) -> ImpactReport {
    let normalized = normalize_root_var(symbol);

    // Group matching occurrences by file.
    use std::collections::BTreeMap;
    let mut per_file: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // file_id -> (safe, refused)
    let mut occurrences_total = 0usize;
    for occ in symbols
        .occurrences()
        .iter()
        .filter(|o| o.normalized_text == normalized)
    {
        occurrences_total += 1;
        let e = per_file.entry(occ.file_id.as_str()).or_insert((0, 0));
        if occ.replace_candidate {
            e.0 += 1;
        } else {
            e.1 += 1;
        }
    }

    let mut impact_files = Vec::new();
    let (mut safe_occurrences, mut refused_occurrences) = (0usize, 0usize);
    for (file_id, (safe, refused)) in &per_file {
        safe_occurrences += safe;
        refused_occurrences += refused;
        let file = files.files().iter().find(|f| f.file_id == *file_id);
        impact_files.push(ImpactFile {
            file_id: (*file_id).to_string(),
            absolute_path: file.map(|f| f.absolute_path.clone()).unwrap_or_default(),
            repo_relative_path: file.and_then(|f| f.repo_relative_path.clone()),
            mutable_policy: file
                .map(|f| f.mutable_policy)
                .unwrap_or(MutablePolicy::ReadOnly),
            occurrence_count: safe + refused,
            safe: *safe,
            refused: *refused,
        });
    }
    impact_files.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));

    let mut definitions: Vec<DbSymbolRow> = symbols
        .symbols()
        .iter()
        .filter(|s| s.normalized_name == normalized)
        .cloned()
        .collect();
    definitions.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));

    ImpactReport {
        symbol: symbol.to_string(),
        normalized_symbol: normalized,
        files_affected: impact_files.len(),
        files: impact_files,
        occurrences_total,
        safe_occurrences,
        refused_occurrences,
        definitions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_index::{FileIndex, ScanScope};
    use std::fs;

    fn tmp(test_name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "envctl-db-symbols-{}-{test_name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn extracts_root_tokens_with_alias_normalization_and_safe_policy() {
        let root = tmp("root-tokens");
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

    #[test]
    fn script_and_config_keys_are_indexed_with_distinct_kinds_and_replace_metadata() {
        let root = tmp("scripts-config");
        std::fs::create_dir_all(root.join("hooks")).unwrap();
        std::fs::create_dir_all(root.join("wrappers")).unwrap();
        std::fs::create_dir_all(root.join("config")).unwrap();

        fs::write(root.join("hooks/pre-commit.sh"), b"#!/bin/sh\n$META_ROOT\n").unwrap();
        fs::write(
            root.join("wrappers/wrapper.sh"),
            b"#!/bin/sh\necho $META_ROOT\n",
        )
        .unwrap();
        fs::write(
            root.join("config/settings.toml"),
            b"[paths]\nroot = \"$META_ROOT\"\n",
        )
        .unwrap();
        fs::write(
            root.join("config/settings.yaml"),
            b"paths:\n  home: /tmp\n  meta: $META_ROOT\n",
        )
        .unwrap();
        fs::write(
            root.join("config/settings.json"),
            b"{\"path\": \"$META_ROOT\", \"tool.path\": \"x\"}\n",
        )
        .unwrap();

        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();

        let hooks = symbols
            .symbols()
            .iter()
            .filter(|s| s.kind == DbSymbolKind::HookScript)
            .collect::<Vec<_>>();
        assert!(!hooks.is_empty());
        assert!(hooks
            .iter()
            .any(|s| s.normalized_name.ends_with("hooks/pre-commit.sh")));

        let wrappers = symbols
            .symbols()
            .iter()
            .filter(|s| s.kind == DbSymbolKind::WrapperScript)
            .collect::<Vec<_>>();
        assert!(!wrappers.is_empty());
        assert!(wrappers
            .iter()
            .any(|s| s.normalized_name.ends_with("wrappers/wrapper.sh")));

        let config_keys = symbols
            .symbols()
            .iter()
            .filter(|s| s.kind == DbSymbolKind::ConfigKey)
            .collect::<Vec<_>>();
        assert!(
            !config_keys.is_empty(),
            "config parser should emit at least one config key"
        );
        assert!(config_keys.iter().any(|s| s.name == "root"));
        assert!(config_keys.iter().any(|s| s.name == "home"));
        assert!(config_keys.iter().any(|s| s.name == "path"));
        assert!(config_keys.iter().any(|s| s.name == "tool.path"));

        let wrapper_occ = symbols
            .occurrences()
            .iter()
            .find(|o| wrappers.iter().any(|s| s.symbol_id == o.symbol_id))
            .unwrap();
        assert!(!wrapper_occ.normalized_text.is_empty());
        assert!(wrapper_occ.replace_candidate);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn impact_maps_blast_radius_split_by_rewrite_safety() {
        let root =
            std::env::temp_dir().join(format!("envctl-db-symbols-impact-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        // Safe (OwnedApply) shell wrapper with two META_ROOT refs.
        fs::write(
            root.join("w.sh"),
            b"cd $META_ROOT/bin\nexport X=${META_ROOT}/x\n",
        )
        .unwrap();
        // Protected .env with a META_ROOT ref -> refused.
        fs::write(root.join(".env"), b"SECRET=$META_ROOT/s\n").unwrap();
        // An unrelated root, to prove filtering.
        fs::write(root.join("other.sh"), b"cd $OTHER_ROOT\n").unwrap();

        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();

        // Alias-normalized lookup: LIFEOS_ROOT would map to LIFE_OS_ROOT; here we
        // ask for META_ROOT directly.
        let report = impact("META_ROOT", &files, &symbols);
        assert_eq!(report.normalized_symbol, "META_ROOT");
        assert_eq!(report.files_affected, 2, "w.sh + .env, not other.sh");
        assert_eq!(report.occurrences_total, 3);
        assert_eq!(report.safe_occurrences, 2, "the two .sh occurrences");
        assert_eq!(report.refused_occurrences, 1, "the .env occurrence");

        let env_file = report
            .files
            .iter()
            .find(|f| f.absolute_path.ends_with(".env"))
            .unwrap();
        assert_eq!(env_file.refused, 1);
        assert_eq!(env_file.safe, 0);
        assert_eq!(env_file.mutable_policy, MutablePolicy::Never);

        // An unknown symbol has an empty, well-formed report (no panic).
        let empty = impact("NOPE_ROOT", &files, &symbols);
        assert_eq!(empty.files_affected, 0);
        assert_eq!(empty.occurrences_total, 0);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn syn_pass_extracts_rust_items_imports_and_clap_derives() {
        let root =
            std::env::temp_dir().join(format!("envctl-db-symbols-syn-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("main.rs"),
            br#"use std::path::PathBuf;
use clap::{Parser, Subcommand};

pub const R: &str = "x";

#[derive(Parser)]
struct Cli {
    name: String,
}

#[derive(Subcommand)]
enum Cmd {
    Roots,
    Query,
}

fn run() -> u32 { 0 }

mod inner {
    pub fn helper() {}
}
"#,
        )
        .unwrap();

        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();

        let rust: Vec<_> = symbols
            .symbols()
            .iter()
            .filter(|s| s.file_id.contains("file:") && s.value.is_some())
            .collect();

        let has = |name: &str, item_kind: &str| {
            rust.iter()
                .any(|s| s.name == name && s.value.as_deref() == Some(item_kind))
        };
        assert!(has("run", "fn"), "fn extracted");
        assert!(has("R", "const"), "const extracted");
        assert!(has("inner", "mod"), "module extracted");
        assert!(has("helper", "fn"), "nested fn extracted");

        // clap derives are surfaced as CliSubcommand.
        let cli = symbols
            .symbols()
            .iter()
            .find(|s| s.name == "Cli")
            .expect("Cli struct");
        assert_eq!(
            cli.kind,
            DbSymbolKind::CliSubcommand,
            "derive(Parser) -> CliSubcommand"
        );
        let cmd = symbols
            .symbols()
            .iter()
            .find(|s| s.name == "Cmd")
            .expect("Cmd enum");
        assert_eq!(
            cmd.kind,
            DbSymbolKind::CliSubcommand,
            "derive(Subcommand) -> CliSubcommand"
        );

        // The nested fn carries its module path.
        let helper = symbols
            .symbols()
            .iter()
            .find(|s| s.name == "helper")
            .unwrap();
        assert_eq!(helper.scope.as_deref(), Some("crate::inner"));

        // `use` imports are captured.
        assert!(
            rust.iter().any(|s| s.value.as_deref() == Some("use")),
            "imports extracted"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn occurrence_byte_spans_are_file_relative_and_slice_exactly() {
        let root =
            std::env::temp_dir().join(format!("envctl-db-symbols-spans-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let content = "π=$OTHER_ROOT\r\ncd ${META_ROOT}/bin\n";
        fs::write(root.join("wrapper.sh"), content).unwrap();

        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();

        for occurrence in symbols.occurrences() {
            assert_eq!(
                &content[occurrence.byte_start..occurrence.byte_end],
                occurrence.match_text
            );
        }
        let meta = symbols
            .occurrences()
            .iter()
            .find(|o| o.normalized_text == "META_ROOT")
            .expect("META_ROOT occurrence");
        assert_eq!(meta.byte_start, content.find("${META_ROOT}").unwrap());
        assert_eq!(meta.byte_end, meta.byte_start + "${META_ROOT}".len());

        let _ = fs::remove_dir_all(&root);
    }
}
