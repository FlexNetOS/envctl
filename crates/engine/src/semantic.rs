//! First envctl ruvector consumer (blueprint R10 / TASK-0094, plan #460 §3
//! row 10, V12+V23): an HNSW top-k index over exported codedb rows, entirely
//! behind the default-OFF `ruvector` cargo feature — default builds compile
//! none of this and the registry pins stop being dead weight.
//!
//! The exported-row contract mirrors the pg `codebase` lane (`module_path` +
//! `embedding_minilm`, the R3 MiniLM 384-d vectors) as emitted by
//! `codedb export --json`: one JSON object per line, extra fields ignored.
//! Engine rules hold: sync, non-printing, no UI — callers render events.

use std::collections::HashMap;
use std::io::BufRead;

use ruvector_core::{DistanceMetric, SearchQuery, VectorDB, VectorEntry};
use serde::Deserialize;

use crate::error::EngineError;

/// One exported codedb row carrying a semantic vector. Extra JSON fields in
/// the export are ignored; rows without a vector are skipped by the loader.
#[derive(Clone, Debug, Deserialize)]
pub struct ExportedRow {
    /// Stable row identity in the export (`module_path` in the codebase lane).
    pub module_path: String,
    /// The MiniLM lane vector (384-d in production; any consistent dim works).
    pub embedding_minilm: Option<Vec<f32>>,
}

/// A top-k answer: the row's `module_path` and its distance score
/// (cosine — lower is closer).
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticHit {
    pub module_path: String,
    pub score: f32,
}

/// In-memory HNSW index over exported codedb rows.
pub struct SemanticIndex {
    db: VectorDB,
    dimensions: usize,
}

impl SemanticIndex {
    /// Build an index over `(module_path, vector)` rows. All vectors must
    /// share one dimension; the first row fixes it.
    pub fn from_rows<I>(rows: I) -> Result<Self, EngineError>
    where
        I: IntoIterator<Item = (String, Vec<f32>)>,
    {
        let mut iter = rows.into_iter().peekable();
        let dimensions = match iter.peek() {
            Some((_, v)) => v.len(),
            None => {
                return Err(EngineError::Semantic(
                    "needs at least one exported row with a vector".into(),
                ))
            }
        };
        let db = VectorDB::with_dimensions(dimensions)
            .map_err(|e| EngineError::Semantic(format!("ruvector VectorDB init: {e}")))?;
        for (module_path, vector) in iter {
            if vector.len() != dimensions {
                return Err(EngineError::Semantic(format!(
                    "row '{module_path}' has dim {} but the index is {dimensions}-d",
                    vector.len()
                )));
            }
            let mut metadata = HashMap::new();
            metadata.insert(
                "module_path".to_string(),
                serde_json::Value::String(module_path.clone()),
            );
            db.insert(VectorEntry {
                id: Some(module_path),
                vector,
                metadata: Some(metadata),
            })
            .map_err(|e| EngineError::Semantic(format!("ruvector insert: {e}")))?;
        }
        Ok(Self { db, dimensions })
    }

    /// Parse `codedb export --json` lines (one JSON row per line) and index
    /// every row that carries an `embedding_minilm` vector.
    pub fn from_export_jsonl<R: BufRead>(reader: R) -> Result<Self, EngineError> {
        let mut rows = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| EngineError::Semantic(format!("export read: {e}")))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let row: ExportedRow = serde_json::from_str(trimmed)
                .map_err(|e| EngineError::Semantic(format!("export row parse: {e}")))?;
            if let Some(vector) = row.embedding_minilm {
                rows.push((row.module_path, vector));
            }
        }
        Self::from_rows(rows)
    }

    /// Answer top-k nearest rows for a query vector (cosine distance,
    /// ascending score).
    pub fn top_k(&self, query: &[f32], k: usize) -> Result<Vec<SemanticHit>, EngineError> {
        if query.len() != self.dimensions {
            return Err(EngineError::Semantic(format!(
                "query dim {} != index dim {}",
                query.len(),
                self.dimensions
            )));
        }
        // Over-fetch then dedup: ruvector-core 2.2.3's HNSW can surface the
        // same node more than once in one result set (observed: identical id +
        // score twice at k=2), so k results straight from the index may hold
        // duplicates. Ascending-score order is preserved; first sighting wins.
        let fetch = k.saturating_mul(4).saturating_add(8);
        let results = self
            .db
            .search(SearchQuery {
                vector: query.to_vec(),
                k: fetch,
                filter: None,
                ef_search: None,
            })
            .map_err(|e| EngineError::Semantic(format!("ruvector search: {e}")))?;
        let mut seen = std::collections::HashSet::new();
        Ok(results
            .into_iter()
            .filter(|r| seen.insert(r.id.clone()))
            .take(k)
            .map(|r| SemanticHit {
                module_path: r.id,
                score: r.score,
            })
            .collect())
    }

    /// Number of indexed rows.
    pub fn len(&self) -> Result<usize, EngineError> {
        self.db
            .len()
            .map_err(|e| EngineError::Semantic(format!("ruvector len: {e}")))
    }

    /// True when the index holds no rows (never constructible today — the
    /// loader requires one row — but keeps the clippy len-without-is-empty
    /// contract honest).
    pub fn is_empty(&self) -> Result<bool, EngineError> {
        Ok(self.len()? == 0)
    }

    /// The vector dimension this index was built with.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// The distance metric in force (cosine by default in ruvector-core).
    pub fn metric(&self) -> DistanceMetric {
        self.db.options().distance_metric
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Fixture shaped exactly like `codedb export --json` lines from the pg
    /// codebase lane (module_path + embedding_minilm; extra fields present).
    const EXPORT_JSONL: &str = r#"
{"module_path":"crates/engine/src/runner.rs","block_type":"file","embedding_minilm":[1.0,0.0,0.0,0.0]}
{"module_path":"crates/engine/src/db.rs","block_type":"file","embedding_minilm":[0.7,0.7,0.0,0.0]}
{"module_path":"docs/ROADMAP.md","block_type":"file","embedding_minilm":[0.0,1.0,0.0,0.0]}
{"module_path":"assets/scripts/install.sh","block_type":"file","embedding_minilm":[0.0,0.0,1.0,0.0]}
{"module_path":"no-vector-row.rs","block_type":"file","embedding_minilm":null}
"#;

    #[test]
    fn top_k_answers_over_exported_rows() {
        let idx = SemanticIndex::from_export_jsonl(Cursor::new(EXPORT_JSONL)).unwrap();
        // The null-vector row is skipped: 4 indexed, 4-d.
        assert_eq!(idx.len().unwrap(), 4);
        assert_eq!(idx.dimensions(), 4);

        // Query almost collinear with runner.rs: runner first, then db.rs
        // (45° away) — angularly separated so cosine order is decisive, not
        // a float-precision tie (first fixture draft had a 6° gap and HNSW
        // legitimately flipped the order).
        let hits = idx.top_k(&[0.98, 0.02, 0.0, 0.0], 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].module_path, "crates/engine/src/runner.rs");
        assert_eq!(hits[1].module_path, "crates/engine/src/db.rs");
        assert!(hits[0].score < hits[1].score);

        // A docs-flavored query surfaces the docs row first.
        let hits = idx.top_k(&[0.0, 0.98, 0.02, 0.0], 1).unwrap();
        assert_eq!(hits[0].module_path, "docs/ROADMAP.md");
    }

    #[test]
    fn dimension_mismatches_are_errors_not_panics() {
        let idx = SemanticIndex::from_export_jsonl(Cursor::new(EXPORT_JSONL)).unwrap();
        assert!(idx.top_k(&[1.0, 0.0], 2).is_err());
        assert!(SemanticIndex::from_rows(vec![
            ("a".to_string(), vec![1.0, 0.0]),
            ("b".to_string(), vec![1.0, 0.0, 0.0]),
        ])
        .is_err());
        assert!(SemanticIndex::from_rows(Vec::<(String, Vec<f32>)>::new()).is_err());
    }
}
