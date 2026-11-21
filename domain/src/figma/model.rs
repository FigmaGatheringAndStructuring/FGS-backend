use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FigmaLink {
    pub file_key: String,
    pub node_id: Option<String>,
}

impl FigmaLink {
    pub fn parse(input: &str) -> Result<Self> {
        let url =
            Url::parse(input.trim()).with_context(|| format!("invalid Figma URL: {input}"))?;

        let host = url.host_str().unwrap_or_default();
        if host != "figma.com" && !host.ends_with(".figma.com") {
            bail!("URL is not a Figma link: {input}");
        }

        let segments: Vec<&str> = url
            .path_segments()
            .map(|segments| segments.collect())
            .unwrap_or_default();

        let kind = segments.first().copied().unwrap_or_default();
        if !matches!(kind, "file" | "design" | "proto" | "board" | "slides") {
            bail!("unsupported Figma URL path: /{kind}");
        }

        let file_key = segments
            .get(1)
            .filter(|key| !key.is_empty())
            .with_context(|| format!("Figma URL is missing a file key: {input}"))?;

        let node_id = url
            .query_pairs()
            .find(|(key, _)| key == "node-id")
            .map(|(_, value)| value.replace('-', ":"));

        Ok(Self {
            file_key: (*file_key).to_string(),
            node_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigmaDocument {
    pub file_key: String,
    pub file_name: String,
    pub last_modified: Option<String>,
    pub node: FigmaNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigmaNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(default)]
    pub children: Vec<FigmaNode>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl FigmaNode {
    /// Reads the `absoluteBoundingBox` property returned by the Figma API.
    pub fn absolute_bounding_box(&self) -> Option<FigmaBoundingBox> {
        self.extra
            .get("absoluteBoundingBox")
            .and_then(FigmaBoundingBox::from_json)
    }

    pub fn is_visible(&self) -> bool {
        self.extra
            .get("visible")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }

    pub fn is_opaque(&self) -> bool {
        self.extra
            .get("opacity")
            .and_then(Value::as_f64)
            .map(|opacity| opacity > 0.0)
            .unwrap_or(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FigmaBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl FigmaBoundingBox {
    pub fn from_json(value: &Value) -> Option<Self> {
        Some(Self {
            x: value.get("x")?.as_f64()?,
            y: value.get("y")?.as_f64()?,
            width: value.get("width")?.as_f64()?,
            height: value.get("height")?.as_f64()?,
        })
    }

    pub fn union(&self, other: &Self) -> Self {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);

        Self {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigmaPageElement {
    pub id: String,
    pub name: String,
    pub element_type: String,
    pub bounding_box: FigmaBoundingBox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigmaPageLayout {
    pub file_key: String,
    pub file_name: String,
    pub page_id: String,
    pub page_name: String,
    /// Outermost renderable nodes used to compose the page PNG. This includes
    /// frames, sections, groups, components and any other top-level scene node.
    pub elements: Vec<FigmaPageElement>,
    /// Every frame on the page, including frames nested inside sections and
    /// groups. This is informational; rendering uses `elements`.
    pub frames: Vec<FigmaPageElement>,
    pub bounding_box: FigmaBoundingBox,
}

impl FigmaPageLayout {
    /// Collects the top-level render units of a page, all frames (including
    /// frames nested inside sections/groups), and computes the combined
    /// bounding box of the render units.
    pub fn from_page(document: &FigmaDocument, page: &FigmaNode) -> Result<Self> {
        let elements = collect_elements(&page.children);
        let frames = collect_frames(&page.children);

        let mut elements_iter = elements.iter();
        let first = elements_iter.next().with_context(|| {
            format!(
                "Figma page '{}' ({}) has no renderable top-level elements",
                page.name, page.id
            )
        })?;
        let bounding_box = elements_iter.fold(first.bounding_box, |acc, element| {
            acc.union(&element.bounding_box)
        });

        Ok(Self {
            file_key: document.file_key.clone(),
            file_name: document.file_name.clone(),
            page_id: page.id.clone(),
            page_name: page.name.clone(),
            elements,
            frames,
            bounding_box,
        })
    }
}

fn element_from_node(node: &FigmaNode) -> Option<FigmaPageElement> {
    let bounding_box = node.absolute_bounding_box()?;
    if bounding_box.width <= 0.0 || bounding_box.height <= 0.0 {
        return None;
    }

    Some(FigmaPageElement {
        id: node.id.clone(),
        name: node.name.clone(),
        element_type: node.element_type.clone(),
        bounding_box,
    })
}

fn collect_elements(nodes: &[FigmaNode]) -> Vec<FigmaPageElement> {
    nodes
        .iter()
        .filter(|node| node.element_type != "SLICE")
        .filter(|node| node.is_visible() && node.is_opaque())
        .filter_map(element_from_node)
        .collect()
}

fn collect_frames(nodes: &[FigmaNode]) -> Vec<FigmaPageElement> {
    let mut frames = Vec::new();
    collect_frames_into(nodes, &mut frames);
    frames
}

fn collect_frames_into(nodes: &[FigmaNode], frames: &mut Vec<FigmaPageElement>) {
    for node in nodes {
        if node.element_type == "FRAME"
            && let Some(frame) = element_from_node(node)
        {
            frames.push(frame);
        }

        collect_frames_into(&node.children, frames);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedImage {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_design_link_with_node_id() {
        let link =
            FigmaLink::parse("https://www.figma.com/design/abc123/My-File?node-id=12-34").unwrap();

        assert_eq!(link.file_key, "abc123");
        assert_eq!(link.node_id.as_deref(), Some("12:34"));
    }

    #[test]
    fn parses_file_link_without_node_id() {
        let link = FigmaLink::parse("https://figma.com/file/abc123/My-File").unwrap();

        assert_eq!(link.file_key, "abc123");
        assert_eq!(link.node_id, None);
    }

    #[test]
    fn rejects_non_figma_links() {
        assert!(FigmaLink::parse("https://example.com/file/abc123").is_err());
    }

    #[test]
    fn unions_bounding_boxes() {
        let a = FigmaBoundingBox {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        let b = FigmaBoundingBox {
            x: 200.0,
            y: 40.0,
            width: 110.0,
            height: 180.0,
        };

        assert_eq!(
            a.union(&b),
            FigmaBoundingBox {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            }
        );
    }

    #[test]
    fn includes_sections_and_nested_frames_in_page_layout() {
        let page: FigmaNode = serde_json::from_value(serde_json::json!({
            "id": "0:1",
            "name": "Page 1",
            "type": "CANVAS",
            "children": [
                {
                    "id": "1:1",
                    "name": "First",
                    "type": "FRAME",
                    "absoluteBoundingBox": { "x": 10, "y": 20, "width": 100, "height": 50 }
                },
                {
                    "id": "1:2",
                    "name": "Section",
                    "type": "SECTION",
                    "absoluteBoundingBox": { "x": 200, "y": 40, "width": 300, "height": 300 },
                    "children": [
                        {
                            "id": "1:3",
                            "name": "Inside section",
                            "type": "FRAME",
                            "absoluteBoundingBox": { "x": 220, "y": 60, "width": 100, "height": 100 }
                        },
                        {
                            "id": "1:4",
                            "name": "Group",
                            "type": "GROUP",
                            "absoluteBoundingBox": { "x": 350, "y": 60, "width": 100, "height": 100 },
                            "children": [
                                {
                                    "id": "1:5",
                                    "name": "Inside group in section",
                                    "type": "FRAME",
                                    "absoluteBoundingBox": { "x": 360, "y": 70, "width": 50, "height": 50 }
                                }
                            ]
                        }
                    ]
                },
                {
                    "id": "1:6",
                    "name": "Hidden",
                    "type": "FRAME",
                    "visible": false,
                    "absoluteBoundingBox": { "x": 500, "y": 500, "width": 10, "height": 10 }
                },
                {
                    "id": "1:7",
                    "name": "Rectangle",
                    "type": "RECTANGLE",
                    "absoluteBoundingBox": { "x": 0, "y": 0, "width": 10, "height": 10 }
                }
            ]
        }))
        .unwrap();

        let document = FigmaDocument {
            file_key: "abc123".to_string(),
            file_name: "Test file".to_string(),
            last_modified: None,
            node: FigmaNode {
                id: "0:0".to_string(),
                name: "Document".to_string(),
                element_type: "DOCUMENT".to_string(),
                children: vec![page.clone()],
                extra: Map::new(),
            },
        };

        let layout = FigmaPageLayout::from_page(&document, &page).unwrap();

        assert_eq!(layout.page_id, "0:1");
        assert_eq!(layout.elements.len(), 3);
        assert_eq!(layout.frames.len(), 4);
        assert_eq!(
            layout.bounding_box,
            FigmaBoundingBox {
                x: 0.0,
                y: 0.0,
                width: 500.0,
                height: 340.0,
            }
        );
        assert!(layout.frames.iter().any(|frame| frame.id == "1:5"));
    }

    #[test]
    fn rejects_page_without_renderable_elements() {
        let page = FigmaNode {
            id: "0:1".to_string(),
            name: "Empty".to_string(),
            element_type: "CANVAS".to_string(),
            children: Vec::new(),
            extra: Map::new(),
        };
        let document = FigmaDocument {
            file_key: "abc123".to_string(),
            file_name: "Test file".to_string(),
            last_modified: None,
            node: FigmaNode {
                id: "0:0".to_string(),
                name: "Document".to_string(),
                element_type: "DOCUMENT".to_string(),
                children: vec![page.clone()],
                extra: Map::new(),
            },
        };

        assert!(FigmaPageLayout::from_page(&document, &page).is_err());
    }
}
