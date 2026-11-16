use async_trait::async_trait;
use domain::figma::model::FigmaDocument;
use domain::figma::repository::FigmaRepository;
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::figma::dto;

const DEFAULT_BASE_URL: &str = "https://api.figma.com/v1";

pub struct FigmaClient {
    http: Client,
    token: String,
    base_url: String,
}

impl FigmaClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            token: token.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        let response = self
            .http
            .get(url)
            .header("X-Figma-Token", &self.token)
            .query(query)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Figma API returned {status}: {body}");
        }

        Ok(response.json::<T>().await?)
    }
}

#[async_trait]
impl FigmaRepository for FigmaClient {
    async fn fetch_file(&self, file_key: &str) -> anyhow::Result<FigmaDocument> {
        let url = format!("{}/files/{}", self.base_url, file_key);
        let response: dto::FileResponse = self.get(&url, &[]).await?;

        Ok(response.into_document(file_key))
    }

    async fn fetch_node(&self, file_key: &str, node_id: &str) -> anyhow::Result<FigmaDocument> {
        let url = format!("{}/files/{}/nodes", self.base_url, file_key);
        let response: dto::NodesResponse = self.get(&url, &[("ids", node_id)]).await?;

        response.into_document(file_key)
    }
}
