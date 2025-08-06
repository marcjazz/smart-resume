use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use aws_sdk_s3::Client as S3Client;
use reqwest::Client as HttpClient;

use crate::{
    config::Config,
    error::AppError,
    models::{ExportRequest, ExportResponse, SummarizeRequest, SummarizeResponse},
    services::{export_pdf, summarize},
};

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

/// Handler for the `/export-pdf` endpoint.
pub async fn export_pdf_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, AppError> {
    let response = export_pdf(payload, &state.s3_client, &state.config).await?;
    Ok(Json(response))
}