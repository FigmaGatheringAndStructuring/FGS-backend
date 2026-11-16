use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FileResponse {
    pub name: String,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    pub document: FigmaNode,
}

#[derive(Debug, Deserialize)]
pub struct NodesResponse {
    pub name: String,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    pub nodes: HashMap<String, NodeEntry>,
}

#[derive(Debug, Deserialize)]
pub struct NodeEntry {
    pub document: FigmaNode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FigmaNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub children: Vec<FigmaNode>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
