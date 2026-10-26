use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Nodes<Node> {
    nodes: Vec<Node>,
}

impl<Node> Nodes<Node> {
    pub fn nodes(&self) -> std::slice::Iter<'_, Node> {
        self.nodes.iter()
    }
}
