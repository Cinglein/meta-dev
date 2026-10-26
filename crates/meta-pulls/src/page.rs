mod page_info;

use serde::Deserialize;

pub use page_info::PageInfo;

#[derive(Clone, Debug, Deserialize)]
#[serde(bound(deserialize = "Node: Deserialize<'de>"))]
pub struct Page<Node> {
    nodes: Vec<Node>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

impl<Node> Page<Node> {
    pub fn nodes(&self) -> std::slice::Iter<'_, Node> {
        self.nodes.iter()
    }

    #[must_use]
    pub fn page_info(&self) -> PageInfo {
        self.page_info
    }
}

impl<Node> Default for Page<Node> {
    fn default() -> Self {
        Self { nodes: Vec::new(), page_info: PageInfo::default() }
    }
}
