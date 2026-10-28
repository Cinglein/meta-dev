use meta_schema::{Actor, Block};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waited {
    block: Block,
    by: Actor,
}

impl Waited {
    pub fn new(block: Block, by: Actor) -> Self {
        Self { block, by }
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn waiter(&self) -> &Actor {
        &self.by
    }
}
