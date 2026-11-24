use crate::ai::dto;
use anyhow::{Context, bail};
use async_trait::async_trait;
use domain::ai::model::{AiRequest, AiResult, Usage};
use domain::ai::usecase::AiUseCase;
use reqwest::Client;
use crate::ai::dto::GenerateRequest;

const DEFAULT_BASE_URL: &str = "https://fgs-ai.juyoungk09.dev";

pub struct AiClient {
    http: Client,
    api_key: String,
    base_url: String,
}

impl AiClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn generate(&self, body: &GenerateRequest<'_>) -> anyhow::Result<AiResult> {
        let url = format!("{}/generate", self.base_url.trim_end_matches('/'));
        let response = self
            .http
            .post(url)
            // .header("X-API-Key", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("failed to call the AI server")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<dto::ErrorResponse>(&body)
                .map(|error| error.error)
                .unwrap_or(body);
            bail!("AI server returned {status}: {message}");
        }

        let response: dto::GenerateResponse = response
            .json()
            .await
            .context("failed to parse the AI response")?;

        Ok(AiResult {
            markdown: response.markdown,
            filtered_elements: response.filtered_elements,
            model: response.model,
            usage: Usage {
                prompt_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.output_tokens,
                elapsed_ms: response.usage.elapsed_ms,
            },
        })
    }
}
