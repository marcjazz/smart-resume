use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;
use my_rust_service::{
    config::Config,
    models::{SummarizeRequest},
    services::summarize::summarize,
};
use reqwest::Client as HttpClient;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Load configuration
    let config = Config::from_env().map_err(|e| Error::from(e.to_string()))?;

    // Initialize clients
    let http_client = HttpClient::new();

    // Parse the request body
    let body = match req.body() {
        Body::Text(s) => s.to_string(),
        Body::Binary(b) => String::from_utf8(b.to_vec()).unwrap_or_default(),
        _ => String::new(),
    };

    let summarize_req: SummarizeRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::Text(format!("Failed to parse request body: {}", e)))?);
        }
    };

    // Call the summarize service
    match summarize(summarize_req, &http_client, &config).await {
        Ok(res) => {
            let json_res = json!(res).to_string();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::Text(json_res))?)
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::Text(e.to_string()))?),
    }
}
