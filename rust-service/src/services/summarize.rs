use crate::{
    config::Config,
    error::AppError,
    models::{Commit, PushEventPayload, SummarizeRequest, SummarizeResponse},
};
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

/// Perform summarization by calling the OpenAI API.
/// Retries once on failure.
pub async fn summarize(
    payload: SummarizeRequest,
    client: &Client,
    config: &Config,
) -> Result<SummarizeResponse, AppError> {
    // Extract commit messages
    let commit_text = payload
        .activities
        .into_iter()
        .filter(|evt| evt.r#type == "PushEvent")
        .filter_map(|evt| {
            serde_json::from_value::<PushEventPayload>(evt.content["payload"].clone()).ok()
        })
        .filter_map(|p| p.commits)
        .flatten()
        .map(|c| c.message)
        .collect::<Vec<_>>()
        .join("\n---\n");

    if commit_text.is_empty() {
        return Ok(SummarizeResponse {
            summary: "No new commits to summarize.".to_string(),
            endpoint_used: "none".to_string(),
        });
    }

    let base_url = payload
        .openapi_base_url
        .or_else(|| Some(config.openapi_base_url.clone()))
        .unwrap();
    let model = payload.model.or_else(|| Some(config.default_model.clone())).unwrap();
    let endpoint = payload
        .endpoint_type
        .or_else(|| Some(config.default_endpoint.clone()))
        .unwrap();
    let api_key = &config.openapi_api_key;

    let url = format!("{}/v1/{}", base_url.trim_end_matches('/'), endpoint);
    let mut last_err = None;

    for attempt in 0..2 {
        let req_body = if endpoint == "embeddings" {
            serde_json::json!({ "model": model, "input": commit_text })
        } else {
            serde_json::json!({ "model": model, "prompt": commit_text })
        };

        let resp = client
            .post(&url)
            .bearer_auth(api_key)
            .json(&req_body)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let v: Value =
                    r.json().await.map_err(|e| AppError::Api(format!("Parse error: {}", e)))?;
                let text = if endpoint == "embeddings" {
                    serde_json::to_string(&v["data"])
                        .map_err(|e| AppError::Api(format!("Serialize error: {}", e)))?
                } else {
                    v["choices"][0]["text"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string()
                };
                return Ok(SummarizeResponse {
                    summary: text,
                    endpoint_used: endpoint,
                });
            }
            Ok(r) => last_err = Some(format!("Upstream API error: {}", r.status())),
            Err(e) => last_err = Some(format!("Request failed: {}", e)),
        }
        sleep(Duration::from_secs(1 << attempt)).await;
    }

    Err(AppError::Api(last_err.unwrap_or_else(|| "Unknown error".to_string())))
}