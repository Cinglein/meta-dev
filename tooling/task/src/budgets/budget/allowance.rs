mod amount;
mod count_part;
mod part;

use serde::{Deserialize, Serialize};

use crate::budgets::budget_id::BudgetId;
use crate::percent::Percent;
use crate::policy::Most;
pub use amount::Amount;
use count_part::CountPart;
use part::Part;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Allowance {
    Whole(Amount),
    PartOf(Part),
    CountOf(CountPart),
}

impl Allowance {
    #[must_use]
    pub fn parent(&self) -> Option<&BudgetId> {
        match self {
            Allowance::PartOf(part) => Some(part.parent()),
            Allowance::CountOf(part) => Some(part.parent()),
            Allowance::Whole(_) => None,
        }
    }

    #[must_use]
    pub fn whole(&self) -> Option<&Amount> {
        match self {
            Allowance::Whole(amount) => Some(amount),
            Allowance::PartOf(_) | Allowance::CountOf(_) => None,
        }
    }

    #[must_use]
    pub fn resolved(&self, parent: impl Fn(&BudgetId) -> Option<Amount>) -> Option<Amount> {
        match self {
            Allowance::Whole(amount) => Some(amount.clone()),
            Allowance::PartOf(part) => {
                parent(part.parent()).and_then(|whole| whole.part(part.percent()))
            }
            Allowance::CountOf(part) => Some(Amount::Count(part.count())),
        }
    }

    #[must_use]
    pub fn share_of(&self, parent: &BudgetId) -> Option<Percent> {
        match self {
            Allowance::PartOf(part) if part.parent() == parent => Some(part.percent()),
            Allowance::Whole(_) | Allowance::PartOf(_) | Allowance::CountOf(_) => None,
        }
    }

    #[must_use]
    pub fn count_of(&self, parent: &BudgetId) -> Option<Most> {
        match self {
            Allowance::CountOf(part) if part.parent() == parent => Some(part.count()),
            Allowance::Whole(_) | Allowance::PartOf(_) | Allowance::CountOf(_) => None,
        }
    }
}
