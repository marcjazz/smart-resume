use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use dotenvy::dotenv;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[derive(Deserialize)]
struct SummarizeRequest {
    github_username: String,
    #[serde(default)]
    openapi_base_url: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    endpoint_type: Option<String>, // "completions" or "embeddings"
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
    commits: Vec<Commit>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
enum Event {
    PushEvent { payload: PushEventPayload },
    #[serde(other)]
    Other,
}

async fn summarize(
    Json(payload): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, (StatusCode, String)> {
    // Fetch GitHub events
    let gh = Client::new();
    let url = format!(
        "https://api.github.com/users/{}/events",
        payload.github_username
    );
    let gh_res = gh
        .get(&url)
        .header("User-Agent", "smart-resume-generator")
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("GitHub request failed: {}", e)))?;
    if !gh_res.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("GitHub API returned: {}", gh_res.status()),
        ));
    }
    let events: Vec<Event> = gh_res.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse GitHub response: {}", e),
        )
    })?;

    // Extract commit messages
    let commit_text = events
        .into_iter()
        .filter_map(|evt| match evt {
            Event::PushEvent { payload } => Some(payload.commits),
            _ => None,
        })
        .flatten()
        .map(|c| c.message)
        .collect::<Vec<_>>()
        .join("\n---\n");

    // Load config
    dotenv().ok();
    let base_url = payload
        .openapi_base_url
        .clone()
        .or_else(|| env::var("OPENAPI_BASE_URL").ok())
        .unwrap_or_else(|| "https://api.openai.com".to_string());
    let model = payload
        .model
        .clone()
        .or_else(|| env::var("DEFAULT_MODEL").ok())
        .unwrap_or_else(|| "text-davinci-003".to_string());
    let endpoint = payload
        .endpoint_type
        .clone()
        .or_else(|| env::var("DEFAULT_ENDPOINT").ok())
        .unwrap_or_else(|| "completions".to_string());
    let api_key =
        env::var("OPENAPI_API_KEY").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "OPENAPI_API_KEY missing".to_string()))?;

    // Prepare HTTP client
    let client = Client::new();
    let url = format!("{}/v1/{}", base_url.trim_end_matches('/'), endpoint);
    let mut last_err = None;

    // Try up to 2 attempts on 429/5xx
    for attempt in 0..2 {
        let req = if endpoint == "embeddings" {
            client
                .post(&url)
                .bearer_auth(&api_key)
                .json(&serde_json::json!({ "model": model, "input": commit_text }))
        } else {
            client
                .post(&url)
                .bearer_auth(&api_key)
                .json(&serde_json::json!({ "model": model, "prompt": commit_text }))
        };
        let resp = req.send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let text = if endpoint == "embeddings" {
                    let v: serde_json::Value = r.json().await.map_err(|e| {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Parse error: {}", e))
                    })?;
                    serde_json::to_string(&v["data"]).map_err(|e| {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialize error: {}", e))
                    })?
                } else {
                    let v: serde_json::Value = r.json().await.map_err(|e| {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Parse error: {}", e))
                    })?;
                    v["choices"][0]["text"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string()
                };
                return Ok(Json(SummarizeResponse {
                    summary: text,
                    endpoint_used: endpoint,
                }));
            }
            Ok(r) if r.status().is_client_error() => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Upstream returned error: {}", r.status()),
                ));
            }
            Ok(r) => {
                last_err = Some(format!("Server error: {}", r.status()));
            }
            Err(e) => {
                last_err = Some(format!("Request failed: {}", e));
            }
        }
        sleep(Duration::from_secs(1 << attempt)).await;
    }

    Err((
        StatusCode::BAD_GATEWAY,
        last_err.unwrap_or_else(|| "Unknown error".to_string()),
    ))
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/summarize", post(summarize));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
