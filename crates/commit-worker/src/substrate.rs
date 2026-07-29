//! Substrate production for the release gates that had no write path.
//!
//! Four gates — `retrieval`, `graph-causal`, `model`, `forecast` — named
//! tables that existed as schema with nothing able to write them, so they
//! measured zero and failed. This module performs the work those gates
//! measure. It is a producer, not a validator: every row it causes to exist
//! is the byproduct of real work, and nothing here writes a
//! `lifeos_release.verification` row.
//!
//! The chain is deliberately one pass, because each stage genuinely needs the
//! previous one's output:
//!
//! 1. Capture real repository documents byte-for-byte into `lifeos_blob`.
//! 2. Embed their chunks through the live native RuvLLM engine — which is a
//!    real model call, so it registers the model and records the invocation
//!    lineage with the exact bytes that crossed the boundary.
//! 3. Record the index generation the embeddings constitute.
//! 4. Build the semantic graph from the measured geometry of those vectors,
//!    and the causal edges from the derivation that actually occurred.
//! 5. Forecast the next embedding batch's latency *before* running it, then
//!    score the prediction against what the batch actually cost.
//!
//! Step 5 is ordered that way on purpose. A forecast written after its own
//! observation proves nothing, so every forecast row here is durably stored
//! before the batch it predicts is executed.

use postgres::Client;
use reqwest::blocking::Client as HttpClient;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::gates::bind_session;
use crate::{connect, CommitError};

/// `postgres::Error` renders as the bare string "db error", which hides the
/// server's message, detail, and the PL/pgSQL context line naming the failing
/// statement. Substrate production drives a dozen database functions, so a
/// failure that does not say which one and why is not diagnosable.
fn db(context: &str, error: postgres::Error) -> CommitError {
    match error.as_db_error() {
        Some(db_error) => CommitError::new(format!(
            "{context}: {} [{}]{}",
            db_error.message(),
            db_error.code().code(),
            db_error
                .detail()
                .map(|detail| format!(" detail: {detail}"))
                .unwrap_or_default()
        )),
        None => CommitError::new(format!("{context}: {error}")),
    }
}

/// The only dimensions migration 0079's projection trigger and
/// `search_embedding` accept. 384 is the smallest, and the engine's native
/// 768-wide output is sliced to it.
const DIMENSION: usize = 384;
/// Bytes per embedded chunk. Small enough that a batch's captured request and
/// response stay comfortably inline in `lifeos_blob`.
const CHUNK_BYTES: usize = 1200;
/// Chunks per embedder call. Each batch is one recorded model invocation.
const BATCH: usize = 8;
/// Samples required before the forecaster will predict anything.
const FORECAST_WARMUP: usize = 3;

/// Source extensions worth indexing. Binary and vendored trees are skipped.
const CORPUS_EXTENSIONS: [&str; 7] = ["md", "rs", "sql", "ts", "svelte", "nu", "toml"];
const SKIP_DIRS: [&str; 8] = [
    "node_modules",
    ".git",
    "target",
    "dist",
    ".kb",
    ".gitnexus",
    "vendor",
    "archives",
];

#[derive(Debug, Serialize)]
pub struct SubstrateReport {
    pub corpus: String,
    pub documents_captured: usize,
    pub chunks_embedded: usize,
    pub dimension: usize,
    pub model_id: String,
    pub model_invocations: usize,
    pub index_generation_id: String,
    pub graph: Value,
    pub causality: Value,
    pub forecasts_issued: usize,
    pub forecasts_scored: usize,
    pub forecast_accuracy: Value,
    pub retrieval_probe: Value,
}

/// Run the whole substrate pass against a live database.
pub fn build(
    conn: &str,
    corpus_root: &Path,
    corpus: &str,
    embedder_url: &str,
    max_documents: usize,
) -> Result<SubstrateReport, CommitError> {
    let mut client = connect(conn)?;
    bind_session(&mut client)?;

    let tenant: String = client
        .query_one("SELECT lifeos_security.current_tenant()::text", &[])
        .map_err(|error| db("resolve bound tenant", error))?
        .get(0);

    let http = HttpClient::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| CommitError::new(format!("embedder client: {error}")))?;

    // 1. Register the engine from what it reports about itself, so the stored
    //    digest is derived from the live service rather than asserted here.
    let (model_id, model_digest) = register_engine(&mut client, &http, embedder_url)?;

    // 2. Capture documents and embed their chunks.
    let documents = discover_documents(corpus_root, max_documents)?;
    if documents.is_empty() {
        return Err(CommitError::new(format!(
            "no indexable documents found under {}",
            corpus_root.display()
        )));
    }

    let mut pending: Vec<Chunk> = Vec::new();
    let mut documents_captured = 0usize;
    for path in &documents {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue; // not UTF-8 text; the embedder takes text only
        };
        if text.trim().is_empty() {
            continue;
        }
        let object_id = store_document(&mut client, &tenant, corpus, path, &text)?;
        documents_captured += 1;
        for (start, end) in chunk_ranges(&text) {
            pending.push(Chunk {
                object_id: object_id.clone(),
                key: format!("{}:{start}-{end}", relative_key(corpus_root, path)),
                start,
                end,
                text: text[start..end].to_string(),
            });
        }
    }

    let mut latencies: Vec<f64> = Vec::new();
    let mut chunks_embedded = 0usize;
    let mut model_invocations = 0usize;
    let mut forecasts: Vec<(String, f64)> = Vec::new();
    let mut scored = 0usize;
    let mut absolute_errors: Vec<f64> = Vec::new();
    let mut inside_interval = 0usize;

    for batch in pending.chunks(BATCH) {
        // Predict this batch before paying for it. The forecast row is durable
        // before the measurement exists.
        let forecast = if latencies.len() >= FORECAST_WARMUP {
            let prediction = predict_next(&latencies);
            let id = issue_forecast(
                &mut client,
                "embedding-batch-latency-ms",
                &latencies,
                &prediction,
            )?;
            forecasts.push((id.clone(), prediction.value));
            Some((id, prediction))
        } else {
            None
        };

        let texts: Vec<&str> = batch.iter().map(|chunk| chunk.text.as_str()).collect();
        let request = json!({ "texts": texts });
        let request_bytes = serde_json::to_vec(&request)
            .map_err(|error| CommitError::new(format!("embed request: {error}")))?;

        let started = Instant::now();
        let response = http
            .post(format!("{}/embed", embedder_url.trim_end_matches('/')))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request_bytes.clone())
            .send()
            .map_err(|error| CommitError::new(format!("embedder request: {error}")))?;
        let status = response.status();
        let body = response
            .bytes()
            .map_err(|error| CommitError::new(format!("embedder response: {error}")))?
            .to_vec();
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        if !status.is_success() {
            return Err(CommitError::new(format!(
                "embedder returned HTTP {status}: {}",
                String::from_utf8_lossy(&body)
            )));
        }

        let parsed: Value = serde_json::from_slice(&body)
            .map_err(|error| CommitError::new(format!("embedder JSON: {error}")))?;
        let vectors = parsed
            .get("vectors")
            .and_then(Value::as_array)
            .ok_or_else(|| CommitError::new("embedder response carries no vectors"))?;
        if vectors.len() != batch.len() {
            return Err(CommitError::new(format!(
                "embedder returned {} vectors for {} chunks",
                vectors.len(),
                batch.len()
            )));
        }

        // The call happened; record it with the exact bytes both ways.
        record_invocation(
            &mut client,
            &model_id,
            &request_bytes,
            &body,
            json!({
                "chunks": batch.len(),
                "latency_ms": elapsed_ms,
                "dimension": DIMENSION,
            }),
        )?;
        model_invocations += 1;

        for (chunk, vector) in batch.iter().zip(vectors) {
            let values = parse_vector(vector)?;
            append_embedding(&mut client, chunk, &values, &model_digest, corpus)?;
            // Same object, same byte range: the lexical and dense halves of
            // hybrid_search must describe one chunk, not two corpora.
            append_lexical(&mut client, chunk, corpus)?;
            chunks_embedded += 1;
        }

        latencies.push(elapsed_ms);

        // Settle the prediction against the cost that actually occurred.
        if let Some((forecast_id, prediction)) = forecast {
            score_forecast(&mut client, &forecast_id, elapsed_ms)?;
            scored += 1;
            absolute_errors.push((elapsed_ms - prediction.value).abs());
            if elapsed_ms >= prediction.lower && elapsed_ms <= prediction.upper {
                inside_interval += 1;
            }
        }
    }

    // 3. The embeddings now constitute an index generation; record the build.
    let index_generation_id: String = client
        .query_one(
            "SELECT lifeos_semantic.record_index_generation($1, $2, $3, $4, $5)::text",
            &[
                &"ruvector-hnsw",
                &corpus,
                &(DIMENSION as i32),
                &json!({ "chunk_bytes": CHUNK_BYTES, "batch": BATCH }),
                &json!({
                    "documents": documents_captured,
                    "chunks": chunks_embedded,
                    "model_invocations": model_invocations,
                }),
            ],
        )
        .map_err(|error| db("record index generation", error))?
        .get(0);

    // 4. Graph from measured geometry, causality from real derivation.
    let graph: Value = client
        .query_one(
            "SELECT lifeos_semantic.build_similarity_graph($1, $2, $3)",
            &[&corpus, &4i32, &0.9f64],
        )
        .map_err(|error| db("build similarity graph", error))?
        .get(0);
    let causality: Value = client
        .query_one(
            "SELECT lifeos_semantic.record_pipeline_causality($1)",
            &[&corpus],
        )
        .map_err(|error| db("record pipeline causality", error))?
        .get(0);

    // 5. Prove the index answers a query, rather than merely existing.
    let retrieval_probe = probe_retrieval(&mut client, &http, embedder_url, corpus)?;

    let mean_absolute_error = if absolute_errors.is_empty() {
        None
    } else {
        Some(absolute_errors.iter().sum::<f64>() / absolute_errors.len() as f64)
    };

    Ok(SubstrateReport {
        corpus: corpus.to_string(),
        documents_captured,
        chunks_embedded,
        dimension: DIMENSION,
        model_id,
        model_invocations,
        index_generation_id,
        graph,
        causality,
        forecasts_issued: forecasts.len(),
        forecasts_scored: scored,
        forecast_accuracy: json!({
            "mean_absolute_error_ms": mean_absolute_error,
            "within_prediction_interval": inside_interval,
            "scored": scored,
        }),
        retrieval_probe,
    })
}

struct Chunk {
    object_id: String,
    key: String,
    start: usize,
    end: usize,
    text: String,
}

/// An EWMA point prediction with a variance-derived 95% interval.
struct Prediction {
    value: f64,
    lower: f64,
    upper: f64,
    alpha: f64,
}

/// Exponentially weighted mean, with the interval taken from the residual
/// spread around that mean. Stated exactly this way in the stored payload —
/// the method is modest, but it is a real forecast and it can be wrong.
fn predict_next(samples: &[f64]) -> Prediction {
    let alpha = 0.4;
    let mut ewma = samples[0];
    for value in &samples[1..] {
        ewma = alpha * value + (1.0 - alpha) * ewma;
    }
    let variance = samples
        .iter()
        .map(|value| (value - ewma).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    let spread = 1.96 * variance.sqrt();
    Prediction {
        value: ewma,
        lower: (ewma - spread).max(0.0),
        upper: ewma + spread,
        alpha,
    }
}

fn issue_forecast(
    client: &mut Client,
    series_key: &str,
    history: &[f64],
    prediction: &Prediction,
) -> Result<String, CommitError> {
    let row = client
        .query_one(
            "SELECT lifeos_agent.issue_forecast($1, $2, $3, $4, $5)::text",
            &[
                &series_key,
                &"ewma-alpha-0.4-with-normal-residual-interval",
                &1i32,
                &json!({ "samples": history, "count": history.len() }),
                &json!({
                    "value": prediction.value,
                    "lower": prediction.lower,
                    "upper": prediction.upper,
                    "alpha": prediction.alpha,
                    "unit": "milliseconds",
                }),
            ],
        )
        .map_err(|error| db("issue forecast", error))?;
    Ok(row.get(0))
}

fn score_forecast(
    client: &mut Client,
    forecast_id: &str,
    observed: f64,
) -> Result<(), CommitError> {
    client
        .query_one(
            "SELECT lifeos_agent.score_forecast($1::text::uuid, $2)::text",
            &[&forecast_id, &observed],
        )
        .map_err(|error| db("score forecast", error))?;
    Ok(())
}

/// Register the engine from its own `/health` response bytes.
fn register_engine(
    client: &mut Client,
    http: &HttpClient,
    embedder_url: &str,
) -> Result<(String, Vec<u8>), CommitError> {
    let health = http
        .get(format!("{}/health", embedder_url.trim_end_matches('/')))
        .send()
        .map_err(|error| {
            CommitError::new(format!(
                "embedder health probe failed; the native RuvLLM engine must be \
                 running for substrate production: {error}"
            ))
        })?;
    if !health.status().is_success() {
        return Err(CommitError::new(format!(
            "embedder health returned HTTP {}",
            health.status()
        )));
    }
    let descriptor = health
        .bytes()
        .map_err(|error| CommitError::new(format!("embedder health body: {error}")))?
        .to_vec();

    let engine: String = serde_json::from_slice::<Value>(&descriptor)
        .ok()
        .and_then(|value| {
            value
                .get("engine")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .ok_or_else(|| CommitError::new("embedder health response names no engine"))?;

    let model_id: String = client
        .query_one(
            "SELECT lifeos_agent.register_model($1, $2, $3, $4)::text",
            &[
                &"ruvllm-native-embedder",
                &engine.as_str(),
                &"text-embedding",
                &descriptor.as_slice(),
            ],
        )
        .map_err(|error| db("register model", error))?
        .get(0);

    // Take the digest PostgreSQL derived, so the embedding rows and the model
    // registry agree by construction rather than by a second computation here.
    let digest_hex: String = client
        .query_one(
            "SELECT typed_payload->>'model_digest' FROM lifeos_agent.model \
             WHERE model_id = $1::text::uuid",
            &[&model_id],
        )
        .map_err(|error| db("read model digest", error))?
        .get(0);
    let digest = decode_hex(&digest_hex)?;

    Ok((model_id, digest))
}

fn record_invocation(
    client: &mut Client,
    model_id: &str,
    input: &[u8],
    output: &[u8],
    statistics: Value,
) -> Result<(), CommitError> {
    client
        .query_one(
            "SELECT lifeos_agent.record_model_invocation($1::text::uuid, $2, $3, $4, $5)::text",
            &[&model_id, &"chunk-embedding", &input, &output, &statistics],
        )
        .map_err(|error| db("record model invocation", error))?;
    Ok(())
}

fn append_embedding(
    client: &mut Client,
    chunk: &Chunk,
    values: &[f32],
    model_digest: &[u8],
    corpus: &str,
) -> Result<(), CommitError> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let literal = format!(
        "[{}]",
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    client
        .query_one(
            "SELECT lifeos_semantic.append_document_embedding(\
                 $1::text::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10)::text",
            &[
                &chunk.object_id,
                &(chunk.start as i64),
                &(chunk.end as i64),
                &(DIMENSION as i32),
                &bytes,
                &literal,
                &model_digest,
                &corpus,
                &chunk.key,
                &json!({ "chunk_bytes": chunk.end - chunk.start }),
            ],
        )
        .map_err(|error| db("append document embedding", error))?;
    Ok(())
}

/// Index the same chunk lexically, so `hybrid_search` fuses two views of one
/// chunk instead of running dense-only over an empty lexical table.
fn append_lexical(client: &mut Client, chunk: &Chunk, corpus: &str) -> Result<(), CommitError> {
    client
        .query_one(
            "SELECT lifeos_semantic.append_lexical_document(\
                 $1::text::uuid, $2, $3, $4, $5, $6)::text",
            &[
                &chunk.object_id,
                &(chunk.start as i64),
                &(chunk.end as i64),
                &chunk.text,
                &corpus,
                &chunk.key,
            ],
        )
        .map_err(|error| db("append lexical document", error))?;
    Ok(())
}

/// Capture the document's exact bytes through the canonical blob ingress.
fn store_document(
    client: &mut Client,
    tenant: &str,
    corpus: &str,
    path: &Path,
    text: &str,
) -> Result<String, CommitError> {
    let row = client
        .query_one(
            "SELECT lifeos_blob.store_bytes($1::text::uuid, $2, $3, $4, $5)::text",
            &[
                &tenant,
                &text.as_bytes(),
                &"text/plain; charset=utf-8",
                &json!({
                    "producer": "lifeos-substrate-corpus",
                    "corpus": corpus,
                    "path": path.display().to_string(),
                }),
                &"substrate-corpus",
            ],
        )
        .map_err(|error| db("store document bytes", error))?;
    Ok(row.get(0))
}

/// Ask the index a real question and report what came back.
fn probe_retrieval(
    client: &mut Client,
    http: &HttpClient,
    embedder_url: &str,
    corpus: &str,
) -> Result<Value, CommitError> {
    let query = "release gate promotion and activation";
    let response = http
        .post(format!("{}/embed", embedder_url.trim_end_matches('/')))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json!({ "texts": [query] }).to_string())
        .send()
        .map_err(|error| CommitError::new(format!("probe embed request: {error}")))?;
    let parsed: Value = serde_json::from_slice(
        &response
            .bytes()
            .map_err(|error| CommitError::new(format!("probe embed body: {error}")))?,
    )
    .map_err(|error| CommitError::new(format!("probe embed JSON: {error}")))?;
    let values = parse_vector(
        parsed
            .get("vectors")
            .and_then(Value::as_array)
            .and_then(|vectors| vectors.first())
            .ok_or_else(|| CommitError::new("probe embedding returned no vector"))?,
    )?;
    let literal = format!(
        "[{}]",
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let branch: String = client
        .query_one(
            "SELECT lifeos_semantic.substrate_branch(lifeos_security.current_tenant())::text",
            &[],
        )
        .map_err(|error| db("resolve branch", error))?
        .get(0);

    let rows = client
        .query(
            "SELECT source_object_id::text, byte_start, byte_end, distance::float8 \
             FROM lifeos_semantic.search_embedding($1::text::extensions.ruvector, \
                                                   $2::text::uuid, 5)",
            &[&literal, &branch],
        )
        .map_err(|error| db("search_embedding probe", error))?;

    let hits: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "source_object_id": row.get::<_, String>(0),
                "byte_start": row.get::<_, i64>(1),
                "byte_end": row.get::<_, i64>(2),
                "distance": row.get::<_, f64>(3),
            })
        })
        .collect();

    // hybrid_search is the surface a caller actually uses. Probe it too, so a
    // dead lexical half cannot hide behind a working dense one.
    let fused = client
        .query(
            "SELECT rank_position, lexical_rank::float8, dense_distance::float8, \
                    fused_rank::float8 \
             FROM lifeos_semantic.hybrid_search($1, $2::text::extensions.ruvector, \
                                                $3::text::uuid, 5)",
            &[&query, &literal, &branch],
        )
        .map_err(|error| db("hybrid_search probe", error))?;

    let fused_hits: Vec<Value> = fused
        .iter()
        .map(|row| {
            json!({
                "rank": row.get::<_, i32>(0),
                "lexical_rank": row.get::<_, Option<f64>>(1),
                "dense_distance": row.get::<_, Option<f64>>(2),
                "fused_rank": row.get::<_, Option<f64>>(3),
            })
        })
        .collect();
    let lexical_contributing = fused_hits
        .iter()
        .filter(|hit| !hit["lexical_rank"].is_null())
        .count();

    Ok(json!({
        "query": query,
        "corpus": corpus,
        "dense_hits": hits.len(),
        "dense_top": hits,
        "hybrid_hits": fused_hits.len(),
        "hybrid_lexical_contributing": lexical_contributing,
        "hybrid_top": fused_hits,
    }))
}

fn parse_vector(value: &Value) -> Result<Vec<f32>, CommitError> {
    let raw = value
        .as_array()
        .ok_or_else(|| CommitError::new("embedding vector is not an array"))?;
    if raw.len() != DIMENSION {
        return Err(CommitError::new(format!(
            "embedder returned {} dimensions; {DIMENSION} required (start the \
             embedder with LIFEOS_RUVLLM_EMBEDDING_DIMENSIONS={DIMENSION})",
            raw.len()
        )));
    }
    raw.iter()
        .map(|component| {
            component
                .as_f64()
                .filter(|value| value.is_finite())
                .map(|value| value as f32)
                .ok_or_else(|| CommitError::new("embedding component is not a finite number"))
        })
        .collect()
}

/// Split on UTF-8 character boundaries so every reported byte range is a
/// valid slice of the exact bytes captured in `lifeos_blob`.
fn chunk_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    while start < text.len() {
        let mut end = (start + CHUNK_BYTES).min(text.len());
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        if text[start..end].trim().is_empty() {
            start = end;
            continue;
        }
        ranges.push((start, end));
        start = end;
    }
    ranges
}

fn relative_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn discover_documents(root: &Path, limit: usize) -> Result<Vec<PathBuf>, CommitError> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|error| CommitError::new(format!("read {}: {error}", dir.display())))?;
        let mut level: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        level.sort();
        for path in level {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            if path.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                    stack.push(path);
                }
                continue;
            }
            let matches = path
                .extension()
                .map(|ext| CORPUS_EXTENSIONS.contains(&ext.to_string_lossy().as_ref()))
                .unwrap_or(false);
            if matches {
                found.push(path);
            }
        }
    }
    found.sort();
    found.truncate(limit);
    Ok(found)
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, CommitError> {
    if !hex.len().is_multiple_of(2) {
        return Err(CommitError::new("model digest hex has odd length"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16)
                .map_err(|_| CommitError::new("model digest is not valid hex"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{chunk_ranges, decode_hex, predict_next, CHUNK_BYTES};

    #[test]
    fn chunk_ranges_cover_the_text_on_character_boundaries() {
        let text = "é".repeat(CHUNK_BYTES); // two bytes per char
        let ranges = chunk_ranges(&text);
        assert!(ranges.len() > 1, "long text must split");
        assert_eq!(ranges[0].0, 0);
        assert_eq!(ranges.last().unwrap().1, text.len());
        for (start, end) in ranges {
            // Panics unless both offsets are real character boundaries.
            let _ = &text[start..end];
        }
    }

    #[test]
    fn chunk_ranges_skip_whitespace_only_regions() {
        assert!(chunk_ranges("   \n\t  ").is_empty());
    }

    #[test]
    fn prediction_interval_brackets_a_steady_series() {
        let prediction = predict_next(&[10.0, 10.0, 10.0, 10.0]);
        assert!((prediction.value - 10.0).abs() < 1e-9);
        assert!(prediction.lower <= 10.0 && prediction.upper >= 10.0);
    }

    #[test]
    fn prediction_tracks_a_rising_series_above_its_first_sample() {
        let prediction = predict_next(&[10.0, 20.0, 30.0, 40.0]);
        assert!(
            prediction.value > 10.0,
            "an exponentially weighted mean must follow the trend"
        );
        assert!(prediction.lower >= 0.0, "latency cannot be negative");
    }

    #[test]
    fn hex_decoding_rejects_malformed_digests() {
        assert_eq!(decode_hex("00ff").unwrap(), vec![0u8, 255u8]);
        assert!(decode_hex("0f0").is_err());
        assert!(decode_hex("zz").is_err());
    }
}
