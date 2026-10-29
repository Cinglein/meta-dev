use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::millis::Millis;
use crate::crate_name::CrateName;
use meta_schema::TopDivision;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutantMillis {
    default: Millis,
    crates: BTreeMap<CrateName, Millis>,
}

impl MutantMillis {
    pub const DIVISION: TopDivision = TopDivision::PerformanceAndCost;

    #[must_use]
    pub fn for_crate(&self, name: &CrateName) -> Millis {
        self.crates
            .get(name)
            .copied()
            .unwrap_or_else(|| self.median())
    }

    #[must_use]
    pub fn is_priced(&self, name: &CrateName) -> bool {
        self.crates.contains_key(name)
    }

    /// The typical measured price: the upper median of the table, so an unpriced
    /// crate is neither the costliest nor the cheapest. `default` only stands in
    /// for an empty table.
    fn median(&self) -> Millis {
        let mut prices: Vec<Millis> = self.crates.values().copied().collect();
        prices.sort_unstable();
        prices
            .get(prices.len() / 2)
            .copied()
            .unwrap_or(self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::{Millis, MutantMillis};
    const TABLE: &str = "default = 5\n\n[crates]\nlisted = 20\nlow = 1\nmid = 9\n";
    const LISTED: &str = "listed";
    const UNLISTED: &str = "unlisted";
    const COSTS: [Millis; 2] = [Millis::new(20), Millis::new(9)];

    #[test]
    fn a_listed_crate_takes_its_own_cost_and_any_other_the_median_of_the_table() {
        let table: MutantMillis = toml::from_str(TABLE).unwrap();
        let costs = [LISTED, UNLISTED].map(|name| table.for_crate(&name.parse().unwrap()));
        assert_eq!(costs, COSTS);
    }
}
