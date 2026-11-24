use domain::figma::model::FigmaPageLayout;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct GenerateRequest<'a> {
    pub layout: &'a FigmaPageLayout,
    pub image: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub markdown: String,
    #[serde(default)]
    pub filtered_elements: Vec<String>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub output_tokens: i64,
    pub elapsed_ms: i64,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
