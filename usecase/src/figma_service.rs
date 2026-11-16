use std::sync::Arc;

use async_trait::async_trait;
use domain::figma::model::{FigmaDocument, FigmaLink};
use domain::figma::repository::FigmaRepository;
use domain::figma::usecase::FigmaUseCase;

pub struct FigmaService {
    repository: Arc<dyn FigmaRepository>,
}

impl FigmaService {
    pub fn new(repository: Arc<dyn FigmaRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl FigmaUseCase for FigmaService {
    async fn get_element(&self, link: &FigmaLink) -> anyhow::Result<FigmaDocument> {
        match link.node_id.as_deref() {
            Some(node_id) => self.repository.fetch_node(&link.file_key, node_id).await,
            None => self.repository.fetch_file(&link.file_key).await,
        }
    }
}
