mod link_count;
mod node_id;

pub use link_count::LinkCount;
pub use node_id::NodeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedNode {
    node: NodeId,
    links: LinkCount,
}

impl SharedNode {
    pub fn new(node: NodeId, links: LinkCount) -> Self {
        Self { node, links }
    }

    pub fn node(self) -> NodeId {
        self.node
    }

    pub fn links(self) -> LinkCount {
        self.links
    }
}
