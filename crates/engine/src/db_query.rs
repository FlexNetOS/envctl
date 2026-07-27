//! db_query — deterministic query surface + agent presets (REQ-054).
//!
//! REQ-050 scaffold: the query AST ([`QueryTable`], [`QueryFilter`],
//! [`QuerySpec`]), the preset enum, and the [`QueryResult`] shape. A minimal,
//! deterministic evaluator (no SQL clone) + `--explain` land in REQ-054.

use crate::db::{Db, DbError, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

impl FromStr for QueryTable {
    type Err = DbError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "files" => Ok(Self::Files),
            "symbols" => Ok(Self::Symbols),
            "occurrences" => Ok(Self::Occurrences),
            "roots" => Ok(Self::Roots),
            "refs" => Ok(Self::Refs),
            "actions" => Ok(Self::Actions),
            other => Err(DbError::Query(format!("unknown table: {other}"))),
        }
    }
}

/// A single deterministic filter clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum QueryFilter {
    Eq {
        field: String,
        value: String,
    },
    Contains {
        field: String,
        value: String,
    },
    In {
        field: String,
        values: Vec<String>,
    },
    PathMatches {
        /// `None` is the `path` alias and checks absolute and relative paths.
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<String>,
        glob: String,
    },
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

impl QueryPreset {
    pub const NAMES: &'static [&'static str] = &[
        "root:meta",
        "root:lifeos",
        "hooks:codex",
        "wrappers:broken",
        "mutable:unsafe",
        "symbols:rust-cli",
        "paths:legacy",
    ];
}

impl FromStr for QueryPreset {
    type Err = DbError;

    fn from_str(value: &str) -> Result<Self> {
        // Keep the scaffold's hyphen spellings as compatibility aliases.
        match value.trim().to_ascii_lowercase().as_str() {
            "root:meta" | "root-meta" => Ok(Self::RootMeta),
            "root:lifeos" | "root-lifeos" => Ok(Self::RootLifeos),
            "hooks:codex" | "hooks-codex" => Ok(Self::HooksCodex),
            "wrappers:broken" | "wrappers-broken" => Ok(Self::WrappersBroken),
            "mutable:unsafe" | "mutable-unsafe" => Ok(Self::MutableUnsafe),
            "symbols:rust-cli" | "symbols-rust-cli" => Ok(Self::SymbolsRustCli),
            "paths:legacy" | "paths-legacy" => Ok(Self::PathsLegacy),
            other => Err(DbError::Query(format!(
                "unknown preset: {other}; expected one of {}",
                Self::NAMES.join(", ")
            ))),
        }
    }
}

impl QueryFilter {
    /// Parse `field==value`, `field contains value`, `field in a,b`, or
    /// `path matches */legacy/*`.
    pub fn parse(expression: &str) -> Result<Self> {
        let expression = expression.trim();
        if let Some((field, value)) = expression.split_once("==") {
            return Ok(Self::Eq {
                field: required(field, expression)?,
                value: required(value, expression)?,
            });
        }
        for (operator, kind) in [(" contains ", 0_u8), (" in ", 1), (" matches ", 2)] {
            if let Some((field, value)) = expression.split_once(operator) {
                let field = required(field, expression)?;
                let value = required(value, expression)?;
                return match kind {
                    0 => Ok(Self::Contains { field, value }),
                    1 => {
                        let values = value
                            .split(',')
                            .map(str::trim)
                            .filter(|item| !item.is_empty())
                            .map(ToOwned::to_owned)
                            .collect::<Vec<_>>();
                        if values.is_empty() {
                            Err(DbError::Query(format!(
                                "filter has an empty 'in' list: {expression}"
                            )))
                        } else {
                            Ok(Self::In { field, values })
                        }
                    }
                    _ if matches!(
                        field.as_str(),
                        "path" | "absolute_path" | "repo_relative_path"
                    ) =>
                    {
                        Ok(Self::PathMatches {
                            field: (field != "path").then_some(field),
                            glob: value,
                        })
                    }
                    _ => Err(DbError::Query(
                        "'matches' is only valid for path fields".into(),
                    )),
                };
            }
        }
        Err(DbError::Query(format!(
            "invalid filter '{expression}'; use ==, contains, in, or matches"
        )))
    }
}

fn required(value: &str, expression: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(DbError::Query(format!(
            "filter contains an empty operand: {expression}"
        )))
    } else {
        Ok(value.to_string())
    }
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

/// Resolve a preset to a concrete (table, filters) pair. Preset names are the
/// stable agent-facing contract.
fn resolve_preset(preset: QueryPreset) -> (QueryTable, Vec<QueryFilter>) {
    use QueryFilter::*;
    match preset {
        QueryPreset::RootMeta => (
            QueryTable::Symbols,
            vec![Eq {
                field: "normalized_name".into(),
                value: "META_ROOT".into(),
            }],
        ),
        QueryPreset::RootLifeos => (
            QueryTable::Symbols,
            vec![Eq {
                field: "normalized_name".into(),
                value: "LIFE_OS_ROOT".into(),
            }],
        ),
        QueryPreset::HooksCodex => (
            QueryTable::Files,
            vec![Contains {
                field: "absolute_path".into(),
                value: "codex".into(),
            }],
        ),
        QueryPreset::WrappersBroken => (
            QueryTable::Files,
            vec![Eq {
                field: "file_kind".into(),
                value: "shell".into(),
            }],
        ),
        QueryPreset::MutableUnsafe => (
            QueryTable::Files,
            vec![Eq {
                field: "mutable_policy".into(),
                value: "never".into(),
            }],
        ),
        QueryPreset::SymbolsRustCli => (
            QueryTable::Files,
            vec![Eq {
                field: "file_kind".into(),
                value: "rust".into(),
            }],
        ),
        QueryPreset::PathsLegacy => (
            QueryTable::Files,
            vec![Contains {
                field: "absolute_path".into(),
                value: "legacy".into(),
            }],
        ),
    }
}

/// Does a JSON row satisfy a single filter? Field lookup is on the row's
/// top-level object keys; missing fields never match.
fn matches(row: &serde_json::Value, filter: &QueryFilter) -> bool {
    let field_str = |field: &str| -> Option<String> {
        row.get(field).map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
    };
    match filter {
        QueryFilter::Eq { field, value } => field_str(field).as_deref() == Some(value.as_str()),
        QueryFilter::Contains { field, value } => {
            field_str(field).map(|s| s.contains(value)).unwrap_or(false)
        }
        QueryFilter::In { field, values } => field_str(field)
            .map(|s| values.contains(&s))
            .unwrap_or(false),
        QueryFilter::PathMatches { field, glob } => match field {
            Some(field) => field_str(field)
                .map(|path| glob_match(glob, &path))
                .unwrap_or(false),
            None => ["absolute_path", "repo_relative_path"]
                .iter()
                .filter_map(|field| field_str(field))
                .any(|path| glob_match(glob, &path)),
        },
    }
}

/// Minimal deterministic glob: supports `*` (any run) segments — enough for the
/// query surface without pulling a glob engine into the hot path.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        match text[pos..].find(part) {
            Some(idx) => {
                if i == 0 && idx != 0 {
                    return false; // no leading '*' -> must anchor at start
                }
                pos += idx + part.len();
            }
            None => return false,
        }
    }
    // No trailing '*' -> must consume to end.
    pattern.ends_with('*') || pos == text.len()
}

/// Evaluate a query against the indexes. Deterministic (rows already sorted by
/// the indexes); `--explain` reports the resolved table + filters. Returns rows
/// as JSON so the `--json` machine contract (REQ-058) stays table-agnostic.
pub fn evaluate(
    spec: &QuerySpec,
    files: &crate::db_index::FileIndex,
    symbols: &crate::db_symbols::SymbolIndex,
) -> Result<QueryResult> {
    let (table, mut filters) = if let Some(preset) = spec.preset {
        let (t, f) = resolve_preset(preset);
        (t, f)
    } else {
        (
            spec.table
                .ok_or_else(|| DbError::Query("query needs a table or a preset".into()))?,
            Vec::new(),
        )
    };
    filters.extend(spec.filters.iter().cloned());

    let all_rows: Vec<serde_json::Value> = match table {
        QueryTable::Files => files
            .files()
            .iter()
            .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
            .collect(),
        QueryTable::Symbols => symbols
            .symbols()
            .iter()
            .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
            .collect(),
        QueryTable::Occurrences => symbols
            .occurrences()
            .iter()
            .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
            .collect(),
        QueryTable::Roots => Db::from_profiles(
            std::env::var("META_ROOT").ok(),
            std::env::var("LIFE_OS_ROOT")
                .or_else(|_| std::env::var("LIFEOS_ROOT"))
                .ok(),
            spec.target_profile.as_deref().unwrap_or("lifeos-release"),
        )
        .roots()
        .iter()
        .map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null))
        .collect(),
        QueryTable::Refs => symbols
            .occurrences()
            .iter()
            .map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null))
            .collect(),
        QueryTable::Actions => symbols
            .occurrences()
            .iter()
            .filter(|row| row.replace_candidate)
            .map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null))
            .collect(),
    };

    let mut rows: Vec<serde_json::Value> = all_rows
        .into_iter()
        .filter(|row| filters.iter().all(|f| matches(row, f)))
        .collect();
    rows.sort_by_key(|row| serde_json::to_string(row).unwrap_or_default());

    let explain = spec.explain.then(|| {
        format!(
            "table={table:?} preset={:?} filters={filters:?} matched={}",
            spec.preset,
            rows.len()
        )
    });

    Ok(QueryResult {
        row_count: rows.len(),
        rows,
        explain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_index::{FileIndex, ScanScope};
    use crate::db_symbols::SymbolIndex;
    use std::fs;

    fn build_indexes() -> (FileIndex, SymbolIndex, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("envctl-db-query-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("cli.rs"), b"const R: &str = \"x\";\n").unwrap();
        fs::write(
            root.join("wrapper.sh"),
            b"cd $META_ROOT\ncd ${LIFEOS_ROOT}\n",
        )
        .unwrap();
        fs::write(root.join(".env"), b"K=$META_ROOT\n").unwrap();
        let files = FileIndex::scan(&ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();
        let symbols = SymbolIndex::build(&files).unwrap();
        (files, symbols, root)
    }

    #[test]
    fn preset_and_filter_queries_are_deterministic_with_explain() {
        let (files, symbols, root) = build_indexes();

        // Preset: symbols:rust-cli -> files where file_kind == rust.
        let spec = QuerySpec {
            table: None,
            filters: vec![],
            preset: Some(QueryPreset::SymbolsRustCli),
            target_profile: None,
            explain: true,
        };
        let res = evaluate(&spec, &files, &symbols).unwrap();
        assert_eq!(res.row_count, 1);
        assert!(res.explain.unwrap().contains("Files"));

        // Preset root:meta -> symbols normalized_name == META_ROOT.
        let spec = QuerySpec {
            table: None,
            filters: vec![],
            preset: Some(QueryPreset::RootMeta),
            target_profile: None,
            explain: false,
        };
        assert_eq!(evaluate(&spec, &files, &symbols).unwrap().row_count, 1);

        // Occurrences table + contains filter on normalized_text.
        let spec = QuerySpec {
            table: Some(QueryTable::Occurrences),
            filters: vec![QueryFilter::Eq {
                field: "normalized_text".into(),
                value: "LIFE_OS_ROOT".into(),
            }],
            preset: None,
            target_profile: None,
            explain: false,
        };
        let res = evaluate(&spec, &files, &symbols).unwrap();
        assert_eq!(res.row_count, 1); // the ${LIFEOS_ROOT} occurrence, normalized

        // A table-less, preset-less query is an error (not a panic).
        let bad = QuerySpec {
            table: None,
            filters: vec![],
            preset: None,
            target_profile: None,
            explain: false,
        };
        assert!(evaluate(&bad, &files, &symbols).is_err());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn glob_match_anchors_correctly() {
        assert!(glob_match("*/wrapper.sh", "/a/b/wrapper.sh"));
        assert!(glob_match("/a/*", "/a/b/c"));
        assert!(!glob_match("/a/*", "/b/c"));
        assert!(glob_match("*.rs", "/x/y/cli.rs"));
        assert!(!glob_match("*.rs", "/x/y/cli.toml"));
    }

    #[test]
    fn parses_contract_presets_tables_and_all_filter_operators() {
        assert_eq!(
            "paths:legacy".parse::<QueryPreset>().unwrap(),
            QueryPreset::PathsLegacy
        );
        assert_eq!(
            "occurrences".parse::<QueryTable>().unwrap(),
            QueryTable::Occurrences
        );
        assert!(matches!(
            QueryFilter::parse("file_kind==rust").unwrap(),
            QueryFilter::Eq { .. }
        ));
        assert!(matches!(
            QueryFilter::parse("absolute_path contains hooks").unwrap(),
            QueryFilter::Contains { .. }
        ));
        assert!(matches!(
            QueryFilter::parse("kind in env_var,rust_item").unwrap(),
            QueryFilter::In { values, .. } if values.len() == 2
        ));
        assert_eq!(
            QueryFilter::parse("path matches */legacy/*").unwrap(),
            QueryFilter::PathMatches {
                field: None,
                glob: "*/legacy/*".into(),
            }
        );
    }

    #[test]
    fn roots_refs_and_actions_tables_are_queryable() {
        let (files, symbols, root) = build_indexes();
        let query = |table| QuerySpec {
            table: Some(table),
            filters: vec![],
            preset: None,
            target_profile: Some("test-release".into()),
            explain: true,
        };
        assert_eq!(
            evaluate(&query(QueryTable::Roots), &files, &symbols)
                .unwrap()
                .row_count,
            2
        );
        let refs = evaluate(&query(QueryTable::Refs), &files, &symbols).unwrap();
        let actions = evaluate(&query(QueryTable::Actions), &files, &symbols).unwrap();
        assert!(refs.row_count >= actions.row_count);
        assert!(refs.explain.unwrap().contains("filters=[]"));
        let _ = fs::remove_dir_all(&root);
    }
}
