use std::collections::BTreeSet;

use task::CrateName;
use task::Millis;
use task::Shard;

#[derive(Clone, Debug)]
pub struct Bin {
    load: Millis,
    crates: BTreeSet<CrateName>,
}

impl Bin {
    pub fn holding(name: CrateName, cost: Millis) -> Bin {
        Bin {
            load: cost,
            crates: BTreeSet::from([name]),
        }
    }

    pub fn fits(&self, cost: Millis, target: Millis) -> bool {
        self.load.plus(cost) <= target
    }

    pub fn take(&mut self, name: CrateName, cost: Millis) {
        self.load = self.load.plus(cost);
        self.crates.insert(name);
    }

    pub fn shard(self) -> Shard {
        Shard::Whole(self.crates)
    }
}
