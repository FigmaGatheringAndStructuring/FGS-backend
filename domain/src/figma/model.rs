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
}
