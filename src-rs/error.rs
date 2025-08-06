use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Unified application error type.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("OpenAI API error: {0}")]
    Api(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tectonic compilation error: {0}")]
    Tectonic(String),

    #[error("S3 upload error: {0}")]
    S3(String),

    #[error("Unexpected error")]
    Unknown,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Api(_) => StatusCode::BAD_GATEWAY,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Tectonic(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::S3(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}