use anyhow::{Result, anyhow};
use domain::figma::model::{FigmaDocument, FigmaNode};

use crate::figma::dto;

impl From<dto::FigmaNode> for FigmaNode {
    fn from(value: dto::FigmaNode) -> Self {
        Self {
            id: value.id,
            name: value.name,
            element_type: value.node_type,
            children: value.children.into_iter().map(Into::into).collect(),
            extra: value.extra,
        }
    }
}

impl dto::FileResponse {
    pub fn into_document(self, file_key: &str) -> FigmaDocument {
        FigmaDocument {
            file_key: file_key.to_string(),
            file_name: self.name,
            last_modified: self.last_modified,
            node: self.document.into(),
        }
    }
}

impl dto::NodesResponse {
    pub fn into_document(self, file_key: &str) -> Result<FigmaDocument> {
        let entry = self
            .nodes
            .into_values()
            .next()
            .ok_or_else(|| anyhow!("Figma returned no nodes for file {file_key}"))?;

        Ok(FigmaDocument {
            file_key: file_key.to_string(),
            file_name: self.name,
            last_modified: self.last_modified,
            node: entry.document.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_nodes_response_to_domain_document() {
        let response: dto::NodesResponse = serde_json::from_value(json!({
            "name": "Test file",
            "lastModified": "2026-01-01T00:00:00Z",
            "nodes": {
                "12:34": {
                    "document": {
                        "id": "12:34",
                        "name": "Button",
                        "type": "FRAME",
                        "absoluteBoundingBox": { "x": 1, "y": 2, "width": 3, "height": 4 },
                        "children": [
                            {
                                "id": "12:35",
                                "name": "Label",
                                "type": "TEXT",
                                "characters": "Hello"
                            }
                        ]
                    }
                }
            }
        }))
        .unwrap();

        let document = response.into_document("abc123").unwrap();

        assert_eq!(document.file_key, "abc123");
        assert_eq!(document.file_name, "Test file");
        assert_eq!(document.node.id, "12:34");
        assert_eq!(document.node.element_type, "FRAME");
        assert_eq!(document.node.children.len(), 1);
        assert_eq!(
            document.node.children[0].extra["characters"],
            json!("Hello")
        );
        assert_eq!(
            document.node.extra["absoluteBoundingBox"]["width"],
            json!(3)
        );
    }
}
