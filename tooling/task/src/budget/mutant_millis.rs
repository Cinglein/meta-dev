use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::millis::Millis;
use crate::crate_name::CrateName;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutantMillis {
    default: Millis,
    crates: BTreeMap<CrateName, Millis>,
}

impl MutantMillis {
    #[must_use]
    pub fn for_crate(&self, name: &CrateName) -> Millis {
        self.crates.get(name).copied().unwrap_or(self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::{Millis, MutantMillis};
    const TABLE: &str = "default = 5\n\n[crates]\nlisted = 9\n";
    const LISTED: &str = "listed";
    const UNLISTED: &str = "unlisted";
    const COSTS: [Millis; 2] = [Millis::new(9), Millis::new(5)];

    #[test]
    fn a_listed_crate_takes_its_own_cost_and_any_other_the_default() {
        let table: MutantMillis = toml::from_str(TABLE).unwrap();
        let costs = [LISTED, UNLISTED].map(|name| table.for_crate(&name.parse().unwrap()));
        assert_eq!(costs, COSTS);
    }
}
