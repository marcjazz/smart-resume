use crate::{
    config::Config,
    error::AppError,
    models::{ExportRequest, ExportResponse},
};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use chrono::Utc;
use std::fs;
use tectonic;

/// Compile LaTeX document and upload PDF to S3.
pub async fn export_pdf(
    payload: ExportRequest,
    s3_client: &S3Client,
    config: &Config,
) -> Result<ExportResponse, AppError> {
    // Load LaTeX template
    let template = fs::read_to_string("rust-service/template.tex")
        .map_err(AppError::Io)?;

    // Populate template
    let latex_doc = template.replace("{{SUMMARY}}", &payload.summary);

    // Compile LaTeX to PDF
    let pdf_data = tectonic::latex_to_pdf(&latex_doc)
        .map_err(|e| AppError::Tectonic(e.to_string()))?;

    // Upload to S3
    let key = format!("resumes/{}.pdf", Utc::now().timestamp_millis());
    s3_client
        .put_object()
        .bucket(&config.aws_s3_bucket_name)
        .key(&key)
        .body(ByteStream::from(pdf_data))
        .content_type("application/pdf")
        .send()
        .await
        .map_err(|e| AppError::S3(e.to_string()))?;

    // Construct URL
    let pdf_url = format!(
        "https://{}.s3.{}.amazonaws.com/{}",
        config.aws_s3_bucket_name, config.aws_region, key
    );

    Ok(ExportResponse { pdf_url })
}