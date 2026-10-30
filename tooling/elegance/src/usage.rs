use std::collections::BTreeSet;

use crate::field_write::FieldWrite;
use crate::usage_node::UsageNode;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    touched: BTreeSet<UsageNode>,
    writes: BTreeSet<FieldWrite>,
}

impl Usage {
    pub fn touch(&mut self, nodes: impl IntoIterator<Item = UsageNode>) {
        self.touched.extend(nodes);
    }

    pub fn write(&mut self, writes: impl IntoIterator<Item = FieldWrite>) {
        self.writes.extend(writes);
    }

    #[must_use]
    pub fn touched(&self) -> &BTreeSet<UsageNode> {
        &self.touched
    }

    #[must_use]
    pub fn writes(&self) -> &BTreeSet<FieldWrite> {
        &self.writes
    }
}
