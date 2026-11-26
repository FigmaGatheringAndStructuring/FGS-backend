use async_trait::async_trait;

use crate::ai::model::AiResult;
use crate::figma::model::FigmaLink;

#[async_trait]
pub trait AiUseCase: Send + Sync {
    async fn get_result(&self, link: &FigmaLink, scale: f64) -> anyhow::Result<AiResult>;
}
