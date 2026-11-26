use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use domain::ai::client::AiClient;
use domain::ai::model::AiResult;
use domain::ai::usecase::AiUseCase;
use domain::figma::model::FigmaLink;
use domain::figma::usecase::FigmaUseCase;
use std::sync::Arc;

pub struct AiService {
    figma: Arc<dyn FigmaUseCase>,
    client: Arc<dyn AiClient>,
}

impl AiService {
    pub fn new(figma: Arc<dyn FigmaUseCase>, client: Arc<dyn AiClient>) -> Self {
        Self { figma, client }
    }
}

#[async_trait]
impl AiUseCase for AiService {
    async fn get_result(&self, link: &FigmaLink, scale: f64) -> anyhow::Result<AiResult> {
        let (layout, image) = self.figma.get_page_and_render(link, scale).await?;
        let image = format!(
            "data:{};base64,{}",
            image.content_type,
            STANDARD.encode(image.bytes)
        );

        self.client.generate(&layout, &image).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ai::client::AiClient as AiClientPort;
    use domain::ai::model::Usage;
    use domain::figma::model::{FigmaBoundingBox, FigmaPageLayout, RenderedImage};
    use std::sync::Mutex;

    struct FakeAiClient {
        seen_image: Mutex<Option<String>>,
    }

    #[async_trait]
    impl AiClientPort for FakeAiClient {
        async fn generate(
            &self,
            layout: &FigmaPageLayout,
            image: &str,
        ) -> anyhow::Result<AiResult> {
            assert_eq!(layout.file_key, "file-key");
            assert_eq!(layout.page_id, "1:2");
            *self.seen_image.lock().unwrap() = Some(image.to_string());

            Ok(AiResult {
                markdown: "# page".to_string(),
                filtered_elements: vec!["1:3".to_string()],
                model: "test-model".to_string(),
                usage: Usage {
                    prompt_tokens: 10,
                    output_tokens: 20,
                    elapsed_ms: 30,
                },
            })
        }
    }

    struct FakeFigma {
        seen_scale: Mutex<Option<f64>>,
    }

    #[async_trait]
    impl FigmaUseCase for FakeFigma {
        async fn get_page(&self, _: &FigmaLink) -> anyhow::Result<FigmaPageLayout> {
            unreachable!("AiService must use get_page_and_render")
        }

        async fn render_page(&self, _: &FigmaLink, _: f64) -> anyhow::Result<RenderedImage> {
            unreachable!("AiService must use get_page_and_render")
        }

        async fn get_page_and_render(
            &self,
            link: &FigmaLink,
            scale: f64,
        ) -> anyhow::Result<(FigmaPageLayout, RenderedImage)> {
            assert_eq!(link.file_key, "file-key");
            assert_eq!(link.node_id.as_deref(), Some("1:2"));
            *self.seen_scale.lock().unwrap() = Some(scale);

            Ok((
                FigmaPageLayout {
                    file_key: "file-key".to_string(),
                    file_name: "File".to_string(),
                    page_id: "1:2".to_string(),
                    page_name: "Page".to_string(),
                    elements: Vec::new(),
                    frames: Vec::new(),
                    bounding_box: FigmaBoundingBox {
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 100.0,
                    },
                },
                RenderedImage {
                    content_type: "image/png".to_string(),
                    bytes: vec![1, 2, 3],
                },
            ))
        }
    }

    fn sample_link() -> FigmaLink {
        FigmaLink {
            file_key: "file-key".to_string(),
            node_id: Some("1:2".to_string()),
        }
    }

    #[tokio::test]
    async fn fetches_the_page_and_sends_it_as_a_data_uri() {
        let figma = Arc::new(FakeFigma {
            seen_scale: Mutex::new(None),
        });
        let client = Arc::new(FakeAiClient {
            seen_image: Mutex::new(None),
        });
        let service = AiService::new(figma.clone(), client.clone());

        let result = service.get_result(&sample_link(), 0.5).await.unwrap();

        assert_eq!(result.markdown, "# page");
        assert_eq!(result.model, "test-model");
        assert_eq!(result.usage.elapsed_ms, 30);
        assert_eq!(figma.seen_scale.lock().unwrap().as_ref(), Some(&0.5));
        assert_eq!(
            client.seen_image.lock().unwrap().as_deref(),
            Some("data:image/png;base64,AQID")
        );
    }
}
