use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub output_tokens: i64,
    pub elapsed_ms: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AiResult {
    pub markdown: String,
    pub filtered_elements: Vec<String>,
    pub model: String,
    pub usage: Usage,
}
