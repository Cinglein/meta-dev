use std::collections::BTreeSet;

use super::recall::Recall;
use crate::Measures;
use crate::smell_inventory::SmellInventory;
use crate::source_path::SourcePath;

#[derive(Clone, Copy, Debug)]
pub struct Score {
    tag: &'static str,
    located: fn(&Measures) -> Vec<BTreeSet<SourcePath>>,
}

impl Score {
    #[must_use]
    pub const fn new(tag: &'static str, located: fn(&Measures) -> Vec<BTreeSet<SourcePath>>) -> Score {
        Score { tag, located }
    }

    #[must_use]
    pub fn recall(&self, measures: &Measures, inventory: &SmellInventory) -> Recall {
        Recall::scored(self.tag, &(self.located)(measures), inventory)
    }
}
