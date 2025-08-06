use axum::{
    extract::State,
    Json,
};
use aws_sdk_s3::Client as S3Client;
use reqwest::Client as HttpClient;

use crate::{
    config::Config,
    error::AppError,
    models::{SummarizeRequest, SummarizeResponse},
};
use crate::services::summarize::summarize;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub http_client: HttpClient,
    pub s3_client: S3Client,
    pub config: Config,
}

/// Handler for the `/summarize` endpoint.
pub async fn summarize_handler(
    State(state): State<AppState>,
    Json(payload): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, AppError> {
    let response = summarize(payload, &state.http_client, &state.config).await?;
    Ok(Json(response))
}
