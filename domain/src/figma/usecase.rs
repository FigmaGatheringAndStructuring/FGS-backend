use async_trait::async_trait;

use crate::figma::model::{FigmaDocument, FigmaLink};

#[async_trait]
pub trait FigmaUseCase: Send + Sync {
    async fn get_element(&self, link: &FigmaLink) -> anyhow::Result<FigmaDocument>;
}
