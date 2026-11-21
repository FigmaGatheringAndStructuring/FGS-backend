use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::figma::model::{FigmaDocument, FigmaLink, FigmaNode, FigmaPageLayout, RenderedImage};
use domain::figma::repository::FigmaRepository;
use domain::figma::usecase::FigmaUseCase;

pub struct FigmaService {
    repository: Arc<dyn FigmaRepository>,
}

impl FigmaService {
    pub fn new(repository: Arc<dyn FigmaRepository>) -> Self {
        Self { repository }
    }

    async fn load_page(&self, link: &FigmaLink) -> Result<FigmaPageLayout> {
        let document = self.repository.fetch_file_depth(&link.file_key, 2).await?;

        if let Some(page) = find_page(&document.node, link.node_id.as_deref()) {
            let page_id = page.id.clone();
            let page_document = self.repository.fetch_node(&link.file_key, &page_id).await?;
            return FigmaPageLayout::from_page(&page_document, &page_document.node);
        }

        if let Some(node_id) = link.node_id.as_deref() {
            let document = self.repository.fetch_file(&link.file_key).await?;
            let page = find_page(&document.node, Some(node_id)).with_context(|| {
                format!(
                    "Figma node {node_id} was not found in file {}",
                    link.file_key
                )
            })?;
            return FigmaPageLayout::from_page(&document, page);
        }

        anyhow::bail!("Figma file {} has no pages", link.file_key)
    }
}

fn find_page<'a>(document: &'a FigmaNode, node_id: Option<&str>) -> Option<&'a FigmaNode> {
    match node_id {
        None => document.children.first(),
        Some(node_id) => document
            .children
            .iter()
            .find(|page| page.id == node_id || contains_node(page, node_id)),
    }
}

fn contains_node(node: &FigmaNode, node_id: &str) -> bool {
    node.children
        .iter()
        .any(|child| child.id == node_id || contains_node(child, node_id))
}

#[async_trait]
impl FigmaUseCase for FigmaService {
    async fn get_element(&self, link: &FigmaLink) -> anyhow::Result<FigmaDocument> {
        match link.node_id.as_deref() {
            Some(node_id) => self.repository.fetch_node(&link.file_key, node_id).await,
            None => self.repository.fetch_file(&link.file_key).await,
        }
    }

    async fn get_page(&self, link: &FigmaLink) -> anyhow::Result<FigmaPageLayout> {
        self.load_page(link).await
    }

    async fn render_page(&self, link: &FigmaLink, scale: f64) -> anyhow::Result<RenderedImage> {
        let page = self.load_page(link).await?;

        self.repository
            .render_elements(&link.file_key, &page.elements, &page.bounding_box, scale)
            .await
    }
}
