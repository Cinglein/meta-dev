mod smell;
mod smell_id;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use settings::EmbeddedDefaults;

use crate::source_path::SourcePath;
use smell::Smell;
pub use smell_id::SmellId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SmellInventory {
    smell: Vec<Smell>,
}

impl EmbeddedDefaults for SmellInventory {
    const DEFAULTS: &str = include_str!("smell_inventory.toml");
}

impl SmellInventory {
    #[must_use]
    pub fn tagged(&self, tag: &str) -> BTreeSet<&SmellId> {
        self.smell.iter().filter(|smell| smell.is_tagged(tag)).map(Smell::smell_id).collect()
    }

    #[must_use]
    pub fn flagged(&self, spots: &BTreeSet<SourcePath>) -> BTreeSet<&SmellId> {
        self.smell.iter().filter(|smell| smell.is_flagged_by(spots)).map(Smell::smell_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use settings::EmbeddedDefaults;

    use super::SmellInventory;

    const LCOM4: &str = "lcom4";
    const LCOM4_ENTRIES: usize = 4;

    #[test]
    fn the_embedded_smell_inventory_parses() {
        assert_eq!(SmellInventory::defaults().tagged(LCOM4).len(), LCOM4_ENTRIES);
    }
}
