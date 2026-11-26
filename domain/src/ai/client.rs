use crate::ai::model::AiResult;
use crate::figma::model::FigmaPageLayout;
use async_trait::async_trait;

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn generate(&self, layout: &FigmaPageLayout, image: &str) -> anyhow::Result<AiResult>;
}
