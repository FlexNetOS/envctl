//! Database-authorized embedding execution for the envctl commit boundary.
//!
//! A staged job may carry an `embedding_request`.  The request is resolved by
//! the configured local `ruvllm-embedder` service before the job is witnessed
//! or inserted into the authoritative table.  Missing capability, malformed
//! vectors, and executor failures fail closed: the cursor cannot advance.

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

use crate::CommitError;

#[derive(Debug, Deserialize)]
struct EmbeddingRequest {
    texts: Vec<String>,
    #[serde(default)]
    expected_dimensions: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    vectors: Vec<Vec<f32>>,
    engine: String,
    corpus_size: usize,
}

/// Enrich one staged job when it requests database-authorized embeddings.
/// Jobs without a request remain byte-for-byte unchanged.
pub fn enrich_job(job_json: &str) -> Result<String, CommitError> {
    let mut job: Value = serde_json::from_str(job_json)
        .map_err(|error| CommitError::new(format!("staged job JSON: {error}")))?;
    let Some(request_value) = job.get("embedding_request").cloned() else {
        if job
            .get("requires_embedding")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && job.get("embedding_result").is_none()
        {
            return Err(CommitError::new(
                "staged job requires embedding but carries no embedding request or result",
            ));
        }
        return Ok(job_json.to_owned());
    };

    let request: EmbeddingRequest = serde_json::from_value(request_value)
        .map_err(|error| CommitError::new(format!("embedding request: {error}")))?;
    if request.texts.is_empty() {
        return Err(CommitError::new(
            "embedding request texts must not be empty",
        ));
    }
    if request.texts.iter().any(|text| text.is_empty()) {
        return Err(CommitError::new(
            "embedding request texts must not contain empty strings",
        ));
    }

    let base = std::env::var("ENVCTL_RUVLLM_EMBEDDER_URL").map_err(|_| {
        CommitError::new("embedding requested but ENVCTL_RUVLLM_EMBEDDER_URL is not configured")
    })?;
    let endpoint = format!("{}/embed", base.trim_end_matches('/'));
    let payload = json!({"texts": request.texts});
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| CommitError::new(format!("embedding client: {error}")))?;
    let response = client
        .post(endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload.to_string())
        .send()
        .map_err(|error| CommitError::new(format!("embedding executor request: {error}")))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| CommitError::new(format!("embedding executor response: {error}")))?;
    if !status.is_success() {
        return Err(CommitError::new(format!(
            "embedding executor returned HTTP {status}: {body}"
        )));
    }

    let result: EmbeddingResponse = serde_json::from_str(&body)
        .map_err(|error| CommitError::new(format!("embedding executor JSON: {error}")))?;
    if result.vectors.len() != request.texts.len() {
        return Err(CommitError::new(format!(
            "embedding executor returned {} vectors for {} texts",
            result.vectors.len(),
            request.texts.len()
        )));
    }
    let dimensions = result
        .vectors
        .first()
        .map(Vec::len)
        .ok_or_else(|| CommitError::new("embedding executor returned no vectors"))?;
    if dimensions == 0 {
        return Err(CommitError::new("embedding vectors must not be empty"));
    }
    if request
        .expected_dimensions
        .is_some_and(|expected| expected != dimensions)
    {
        return Err(CommitError::new(format!(
            "embedding dimension {dimensions} does not match requested {}",
            request.expected_dimensions.unwrap_or_default()
        )));
    }
    if result
        .vectors
        .iter()
        .any(|vector| vector.len() != dimensions || vector.iter().any(|value| !value.is_finite()))
    {
        return Err(CommitError::new(
            "embedding vectors have inconsistent dimensions or non-finite values",
        ));
    }

    let object = job
        .as_object_mut()
        .ok_or_else(|| CommitError::new("staged embedding job must be a JSON object"))?;
    object.insert(
        "embedding_result".to_string(),
        json!({
            "engine": result.engine,
            "corpus_size": result.corpus_size,
            "dimensions": dimensions,
            "vectors": result.vectors,
        }),
    );
    object.insert("requires_embedding".to_string(), Value::Bool(false));
    serde_json::to_string(&job)
        .map_err(|error| CommitError::new(format!("serialize enriched job: {error}")))
}

#[cfg(test)]
mod tests {
    use super::enrich_job;

    #[test]
    fn leaves_jobs_without_embedding_requests_unchanged() {
        let job = r#"{"seq":7,"payload":"raw"}"#;
        assert_eq!(enrich_job(job).unwrap(), job);
    }

    #[test]
    fn fails_closed_when_embedding_capability_is_missing() {
        let job = r#"{"requires_embedding":true,"embedding_request":{"texts":["hello"]}}"#;
        std::env::remove_var("ENVCTL_RUVLLM_EMBEDDER_URL");
        let error = enrich_job(job).unwrap_err().to_string();
        assert!(error.contains("ENVCTL_RUVLLM_EMBEDDER_URL"));
    }

    #[test]
    fn fails_closed_for_required_embedding_without_request_or_result() {
        let error = enrich_job(r#"{"requires_embedding":true}"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("no embedding request or result"));
    }

    #[test]
    fn enriches_through_the_live_executor_when_configured() {
        let Ok(_) = std::env::var("ENVCTL_RUVLLM_EMBEDDER_URL") else {
            return;
        };
        let enriched = enrich_job(
            r#"{"requires_embedding":true,"embedding_request":{"texts":["envctl live embedding"],"expected_dimensions":128}}"#,
        )
        .expect("configured ruvllm-embedder must enrich the staged job");
        let value: serde_json::Value = serde_json::from_str(&enriched).unwrap();
        assert_eq!(value["requires_embedding"], false);
        assert_eq!(value["embedding_result"]["dimensions"], 128);
        assert_eq!(
            value["embedding_result"]["vectors"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
