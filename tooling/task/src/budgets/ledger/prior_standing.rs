use std::collections::BTreeMap;
use std::str::FromStr;

use serde::de::Error;
use serde::{Deserialize, Deserializer};

use super::super::{BudgetDrafts, Budgets};

#[derive(Debug, Default, Deserialize)]
pub struct PriorStanding {
    #[serde(default, deserialize_with = "PriorStanding::standing_only")]
    budgets: Budgets,
}

impl PriorStanding {
    fn standing_only<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Budgets, D::Error> {
        let kept: BTreeMap<_, _> = BTreeMap::from(BudgetDrafts::deserialize(deserializer)?)
            .into_iter()
            .filter(|(_, budget)| budget.standing_in().is_some())
            .collect();
        Budgets::try_from(kept).map_err(D::Error::custom)
    }

    #[must_use]
    pub fn budgets(&self) -> &Budgets {
        &self.budgets
    }
}

impl FromStr for PriorStanding {
    type Err = toml::de::Error;

    fn from_str(text: &str) -> Result<PriorStanding, toml::de::Error> {
        toml::from_str(text)
    }
}
