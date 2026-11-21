use async_trait::async_trait;

use crate::figma::model::{FigmaBoundingBox, FigmaDocument, FigmaPageElement, RenderedImage};

#[async_trait]
pub trait FigmaRepository: Send + Sync {
    async fn fetch_file(&self, file_key: &str) -> anyhow::Result<FigmaDocument>;
    async fn fetch_node(&self, file_key: &str, node_id: &str) -> anyhow::Result<FigmaDocument>;

    async fn fetch_file_depth(&self, file_key: &str, depth: u32) -> anyhow::Result<FigmaDocument>;

    async fn render_elements(
        &self,
        file_key: &str,
        elements: &[FigmaPageElement],
        bounds: &FigmaBoundingBox,
        scale: f64,
    ) -> anyhow::Result<RenderedImage>;
}
