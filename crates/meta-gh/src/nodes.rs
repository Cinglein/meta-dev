use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(bound(deserialize = "Node: Deserialize<'de>"))]
pub struct Nodes<Node> {
    #[serde(default)]
    nodes: Vec<Node>,
}

impl<Node> Nodes<Node> {
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}

impl<Node> Default for Nodes<Node> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}
