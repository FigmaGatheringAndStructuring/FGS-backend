use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use domain::figma::model::{FigmaBoundingBox, FigmaDocument, FigmaPageElement, RenderedImage};
use domain::figma::repository::FigmaRepository;
use image::{ImageFormat, RgbaImage, imageops};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::figma::dto;

const DEFAULT_BASE_URL: &str = "https://api.figma.com/v1";
const RENDER_BATCH_SIZE: usize = 100;
const MAX_RENDER_PIXELS: u64 = 32_000_000;

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

    async fn download_image(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.http.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("failed to download rendered image ({status}): {body}");
        }

        Ok(response.bytes().await?.to_vec())
    }

    async fn render_batch(
        &self,
        file_key: &str,
        elements: &[FigmaPageElement],
        scale: f64,
    ) -> Result<HashMap<String, Vec<u8>>> {
        let ids = elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let scale_value = scale.to_string();
        let url = format!("{}/images/{}", self.base_url, file_key);

        let response: dto::ImagesResponse = self
            .get(
                &url,
                &[
                    ("ids", ids.as_str()),
                    ("format", "png"),
                    ("scale", scale_value.as_str()),
                    ("use_absolute_bounds", "true"),
                ],
            )
            .await?;

        if let Some(error) = response.err.as_deref() {
            bail!("Figma image API returned an error: {error}");
        }
        if response.images.is_empty() {
            bail!(
                "Figma image API returned no images (status: {:?})",
                response.status
            );
        }

        let mut rendered = HashMap::with_capacity(elements.len());
        for element in elements {
            let image_url = response
                .images
                .get(&element.id)
                .and_then(|url| url.as_deref())
                .with_context(|| {
                    format!(
                        "Figma could not render element '{}' ({}, {})",
                        element.name, element.element_type, element.id
                    )
                })?;

            rendered.insert(element.id.clone(), self.download_image(image_url).await?);
        }

        Ok(rendered)
    }
}

fn compose(
    elements: &[FigmaPageElement],
    rendered: &HashMap<String, Vec<u8>>,
    bounds: &FigmaBoundingBox,
    scale: f64,
) -> Result<RenderedImage> {
    let width = ((bounds.width * scale).round() as i64).max(1) as u32;
    let height = ((bounds.height * scale).round() as i64).max(1) as u32;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_RENDER_PIXELS {
        bail!("rendered image would be {width}x{height} ({pixels} pixels); reduce scale");
    }

    let mut canvas = RgbaImage::new(width, height);

    for element in elements {
        let bytes = rendered
            .get(&element.id)
            .with_context(|| format!("element '{}' was not rendered", element.name))?;
        let image = image::load_from_memory(bytes)
            .with_context(|| {
                format!(
                    "failed to decode rendered element '{}' ({})",
                    element.name, element.element_type
                )
            })?
            .to_rgba8();

        let x = ((element.bounding_box.x - bounds.x) * scale).round() as i64;
        let y = ((element.bounding_box.y - bounds.y) * scale).round() as i64;
        imageops::overlay(&mut canvas, &image, x, y);
    }

    let mut bytes = Vec::new();
    canvas
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .context("failed to encode composed PNG")?;

    Ok(RenderedImage {
        content_type: "image/png".to_string(),
        bytes,
    })
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

    async fn fetch_file_depth(&self, file_key: &str, depth: u32) -> anyhow::Result<FigmaDocument> {
        let url = format!("{}/files/{}", self.base_url, file_key);
        let depth = depth.to_string();
        let response: dto::FileResponse = self.get(&url, &[("depth", depth.as_str())]).await?;

        Ok(response.into_document(file_key))
    }

    async fn render_elements(
        &self,
        file_key: &str,
        elements: &[FigmaPageElement],
        bounds: &FigmaBoundingBox,
        scale: f64,
    ) -> anyhow::Result<RenderedImage> {
        if elements.is_empty() {
            bail!("no page elements to render");
        }
        if !(0.01..=4.0).contains(&scale) {
            bail!("scale must be between 0.01 and 4.0");
        }

        let width = ((bounds.width * scale).round() as i64).max(1) as u32;
        let height = ((bounds.height * scale).round() as i64).max(1) as u32;
        let pixels = u64::from(width) * u64::from(height);
        if pixels > MAX_RENDER_PIXELS {
            bail!("rendered image would be {width}x{height} ({pixels} pixels); reduce scale");
        }

        let mut rendered = HashMap::with_capacity(elements.len());
        for chunk in elements.chunks(RENDER_BATCH_SIZE) {
            rendered.extend(self.render_batch(file_key, chunk, scale).await?);
        }

        compose(elements, &rendered, bounds, scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        let image = RgbaImage::from_pixel(width, height, Rgba(color));
        let mut bytes = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn composes_elements_at_absolute_positions() {
        let elements = vec![
            FigmaPageElement {
                id: "1:1".to_string(),
                name: "left".to_string(),
                element_type: "FRAME".to_string(),
                bounding_box: FigmaBoundingBox {
                    x: 10.0,
                    y: 20.0,
                    width: 10.0,
                    height: 10.0,
                },
            },
            FigmaPageElement {
                id: "1:2".to_string(),
                name: "right".to_string(),
                element_type: "SECTION".to_string(),
                bounding_box: FigmaBoundingBox {
                    x: 30.0,
                    y: 20.0,
                    width: 10.0,
                    height: 10.0,
                },
            },
        ];
        let bounds = FigmaBoundingBox {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 10.0,
        };

        let mut rendered = HashMap::new();
        rendered.insert("1:1".to_string(), png(10, 10, [255, 0, 0, 255]));
        rendered.insert("1:2".to_string(), png(10, 10, [0, 0, 255, 255]));

        let result = compose(&elements, &rendered, &bounds, 1.0).unwrap();
        let image = image::load_from_memory(&result.bytes).unwrap().to_rgba8();

        assert_eq!(image.dimensions().0, 30);
        assert_eq!(image.dimensions().1, 10);
        assert_eq!(image.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
        assert_eq!(image.get_pixel(20, 0), &Rgba([0, 0, 255, 255]));
        assert_eq!(image.get_pixel(10, 0), &Rgba([0, 0, 0, 0]));
    }
}
