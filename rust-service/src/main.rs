mod config;
mod models;
mod error;
mod services;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
    extract::State,
};
use aws_config;
use aws_sdk_s3::Client as S3Client;
use reqwest::Client as HttpClient;
use tokio;
use crate::{
    config::Config,
    error::AppError,
    handlers::{AppState, summarize_handler, export_pdf_handler},
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize clients
    let http_client = HttpClient::new();
    let aws_conf = aws_config::load_from_env().await;
    let s3_client = S3Client::new(&aws_conf);

    let state = AppState {
        http_client,
        s3_client,
        config,
    };

    // Build application with state
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/summarize", post(summarize_handler))
        .route("/export-pdf", post(export_pdf_handler))
        .with_state(state);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(AppError::Io)?;

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await;

    Ok(())
}
