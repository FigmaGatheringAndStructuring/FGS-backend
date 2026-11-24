use async_trait::async_trait;

use crate::figma::model::{FigmaLink, FigmaPageLayout, RenderedImage};

#[async_trait]
pub trait FigmaUseCase: Send + Sync {
    async fn get_page(&self, link: &FigmaLink) -> anyhow::Result<FigmaPageLayout>;

    async fn render_page(&self, link: &FigmaLink, scale: f64) -> anyhow::Result<RenderedImage>;
}
