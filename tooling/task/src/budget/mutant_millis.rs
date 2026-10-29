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
    pub fn unpriced_note(&self, name: &CrateName) -> Option<String> {
        (!self.crates.contains_key(name)).then(|| {
            format!(
                "mutants: {name} is unpriced in [mutant_millis]; planned at the table median, {} ms per mutant; `cargo xtask mutant-costs` will measure it",
                self.median()
            )
        })
    }

    fn median(&self) -> Millis {
        let mut prices: Vec<Millis> = self.crates.values().copied().collect();
        prices.sort_unstable();
        prices
            .get(prices.len() >> 1)
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
    const NOTE: &str = "mutants: unlisted is unpriced in [mutant_millis]; planned at the table median, 9 ms per mutant; `cargo xtask mutant-costs` will measure it";
    const COSTS: [Millis; 2] = [Millis::new(20), Millis::new(9)];

    #[test]
    fn a_listed_crate_takes_its_own_cost_and_any_other_the_median_of_the_table() {
        let table: MutantMillis = toml::from_str(TABLE).unwrap();
        let costs = [LISTED, UNLISTED].map(|name| table.for_crate(&name.parse().unwrap()));
        assert_eq!(costs, COSTS);
    }

    #[test]
    fn only_an_unlisted_crate_gets_a_note_naming_it_its_price_and_the_command_that_measures_it() {
        let table: MutantMillis = toml::from_str(TABLE).unwrap();
        let notes = [LISTED, UNLISTED].map(|name| table.unpriced_note(&name.parse().unwrap()));
        assert_eq!(notes, [None, Some(NOTE.to_owned())]);
    }
}
