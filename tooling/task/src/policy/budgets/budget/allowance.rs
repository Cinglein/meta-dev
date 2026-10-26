mod amount;
mod part;

use serde::{Deserialize, Serialize};

use crate::policy::budgets::budget_id::BudgetId;
use crate::policy::percent::Percent;
use amount::Amount;
use part::Part;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Allowance {
    Whole(Amount),
    PartOf(Part),
}

impl Allowance {
    #[must_use]
    pub fn part(&self) -> Option<&Part> {
        match self {
            Allowance::PartOf(part) => Some(part),
            Allowance::Whole(_) => None,
        }
    }

    #[must_use]
    pub fn share_of(&self, parent: &BudgetId) -> Option<Percent> {
        self.part()
            .filter(|part| part.parent() == parent)
            .map(Part::percent)
    }
}
