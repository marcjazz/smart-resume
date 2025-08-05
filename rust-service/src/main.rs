use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use dotenvy::dotenv;
use reqwest::Client;
use tokio::time::{sleep, Duration};
use aws_sdk_s3::primitives::ByteStream;
use tectonic;

// SECTION: /summarize endpoint
#[derive(Deserialize)]
struct Activity {
    r#type: String,
    content: Value,
}

#[derive(Deserialize)]
struct SummarizeRequest {
    activities: Vec<Activity>,
    #[serde(default)]
    openapi_base_url: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    endpoint_type: Option<String>,
}

#[derive(Serialize)]
struct SummarizeResponse {
    summary: String,
    endpoint_used: String,
}

#[derive(Deserialize)]
struct Commit {
    message: String,
}

#[derive(Deserialize)]
struct PushEventPayload {
    commits: Option<Vec<Commit>>,
}

async fn summarize(
    Json(payload): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, (StatusCode, String)> {
    let commit_text = payload.activities
        .iter()
        .filter(|evt| evt.r#type == "PushEvent")
        .filter_map(|evt| serde_json::from_value::<PushEventPayload>(evt.content["payload"].clone()).ok())
        .filter_map(|p| p.commits)
        .flatten()
        .map(|c| c.message)
        .collect::<Vec<_>>()
        .join("\n---\n");

    if commit_text.is_empty() {
        return Ok(Json(SummarizeResponse {
            summary: "No new commits to summarize.".to_string(),
            endpoint_used: "none".to_string(),
        }));
    }

    dotenv().ok();
    let base_url = payload.openapi_base_url.or_else(|| env::var("OPENAPI_BASE_URL").ok()).unwrap_or_else(|| "https://api.openai.com".to_string());
    let model = payload.model.or_else(|| env::var("DEFAULT_MODEL").ok()).unwrap_or_else(|| "text-davinci-003".to_string());
    let endpoint = payload.endpoint_type.or_else(|| env::var("DEFAULT_ENDPOINT").ok()).unwrap_or_else(|| "completions".to_string());
    let api_key = env::var("OPENAPI_API_KEY").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "OPENAPI_API_KEY missing".to_string()))?;

    let client = Client::new();
    let url = format!("{}/v1/{}", base_url.trim_end_matches('/'), endpoint);
    let mut last_err = None;

    for attempt in 0..2 {
        let req_body = if endpoint == "embeddings" {
            serde_json::json!({ "model": model, "input": commit_text })
        } else {
            serde_json::json!({ "model": model, "prompt": commit_text })
        };
        let resp = client.post(&url).bearer_auth(&api_key).json(&req_body).send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let v: Value = r.json().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Parse error: {}", e)))?;
                let text = if endpoint == "embeddings" {
                    serde_json::to_string(&v["data"]).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialize error: {}", e)))?
                } else {
                    v["choices"][0]["text"].as_str().unwrap_or_default().to_string()
                };
                return Ok(Json(SummarizeResponse { summary: text, endpoint_used: endpoint }));
            }
            Ok(r) => last_err = Some(format!("Upstream API error: {}", r.status())),
            Err(e) => last_err = Some(format!("Request failed: {}", e)),
        }
        sleep(Duration::from_secs(1 << attempt)).await;
    }
    Err((StatusCode::BAD_GATEWAY, last_err.unwrap_or_else(|| "Unknown error".to_string())))
}


// SECTION: /export-pdf endpoint
#[derive(Deserialize)]
struct ExportRequest {
    summary: String,
}

#[derive(Serialize)]
struct ExportResponse {
    pdf_url: String,
}

async fn export_pdf(
    Json(payload): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, (StatusCode, String)> {
    // 1. Load LaTeX template
    let template = std::fs::read_to_string("rust-service/template.tex")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read LaTeX template: {}", e)))?;

    // 2. Populate template
    let latex_doc = template.replace("{{SUMMARY}}", &payload.summary);

    // 3. Compile LaTeX to PDF using Tectonic
    let pdf_data = tectonic::latex_to_pdf(&latex_doc)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tectonic compilation failed: {}", e.to_string())))?;

    // 4. Upload to S3
    let aws_config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let bucket = env::var("AWS_S3_BUCKET_NAME").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "AWS_S3_BUCKET_NAME not set".to_string()))?;
    let key = format!("resumes/{}.pdf", chrono::Utc::now().timestamp_millis());

    s3_client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .body(ByteStream::from(pdf_data))
        .content_type("application/pdf")
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to upload to S3: {}", e)))?;

    // 5. Construct and return public URL
    let region = env::var("AWS_REGION").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "AWS_REGION not set".to_string()))?;
    let pdf_url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, key);

    Ok(Json(ExportResponse { pdf_url }))
}

// SECTION: Main app
#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/summarize", post(summarize))
        .route("/export-pdf", post(export_pdf));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
