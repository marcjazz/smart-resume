use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a GitHub activity event.
#[derive(Deserialize)]
pub struct Activity {
    pub r#type: String,
    pub content: Value,
}

/// Request body for the `/summarize` endpoint.
#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub activities: Vec<Activity>,
    #[serde(default)]
    pub openapi_base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub endpoint_type: Option<String>,
}

/// Response body for the `/summarize` endpoint.
#[derive(Serialize)]
pub struct SummarizeResponse {
    pub summary: String,
    pub endpoint_used: String,
}

/// Represents a commit entry in a push event payload.
#[derive(Deserialize)]
pub struct Commit {
    pub message: String,
}

/// Payload for a GitHub push event.
#[derive(Deserialize)]
pub struct PushEventPayload {
    pub commits: Option<Vec<Commit>>,
}

/// Request body for the `/export-pdf` endpoint.
#[derive(Deserialize)]
pub struct ExportRequest {
    pub summary: String,
}

/// Response body for the `/export-pdf` endpoint.
#[derive(Serialize)]
pub struct ExportResponse {
    pub pdf_url: String,
}