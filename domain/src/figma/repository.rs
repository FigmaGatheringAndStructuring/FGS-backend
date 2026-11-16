use async_trait::async_trait;

use crate::figma::model::FigmaDocument;

#[async_trait]
pub trait FigmaRepository: Send + Sync {
    async fn fetch_file(&self, file_key: &str) -> anyhow::Result<FigmaDocument>;
    async fn fetch_node(&self, file_key: &str, node_id: &str) -> anyhow::Result<FigmaDocument>;
}
