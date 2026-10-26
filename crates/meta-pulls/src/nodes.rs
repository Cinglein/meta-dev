use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Nodes<Node> {
    nodes: Vec<Node>,
}

impl<Node> Nodes<Node> {
    pub fn iter(&self) -> std::slice::Iter<'_, Node> {
        self.nodes.iter()
    }
}

impl<'nodes, Node> IntoIterator for &'nodes Nodes<Node> {
    type Item = &'nodes Node;
    type IntoIter = std::slice::Iter<'nodes, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
