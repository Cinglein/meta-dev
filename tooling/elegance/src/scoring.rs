use std::fmt;

use crate::Measures;
use crate::catalog::Catalog;
use crate::smell_inventory::SmellInventory;

#[derive(Debug)]
pub struct Scoring<'measured> {
    measures: &'measured Measures,
    inventory: &'measured SmellInventory,
}

impl<'measured> Scoring<'measured> {
    #[must_use]
    pub fn new(measures: &'measured Measures, inventory: &'measured SmellInventory) -> Scoring<'measured> {
        Scoring { measures, inventory }
    }
}

impl fmt::Display for Scoring<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Catalog::scored(self.measures, self.inventory, formatter)
    }
}
