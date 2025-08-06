use std::env;
use dotenvy::dotenv;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Config {
    pub openapi_base_url: String,
    pub default_model: String,
    pub default_endpoint: String,
    pub openapi_api_key: String,
    pub aws_s3_bucket_name: String,
    pub aws_region: String,
}

impl Config {
    /// Load configuration from environment variables, using defaults where applicable.
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let openapi_base_url = env::var("OPENAPI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());
        let default_model = env::var("DEFAULT_MODEL")
            .unwrap_or_else(|_| "text-davinci-003".to_string());
        let default_endpoint = env::var("DEFAULT_ENDPOINT")
            .unwrap_or_else(|_| "completions".to_string());
        let openapi_api_key = env::var("OPENAPI_API_KEY")?;
        let aws_s3_bucket_name = env::var("AWS_S3_BUCKET_NAME")?;
        let aws_region = env::var("AWS_REGION")?;

        Ok(Config {
            openapi_base_url,
            default_model,
            default_endpoint,
            openapi_api_key,
            aws_s3_bucket_name,
            aws_region,
        })
    }
}