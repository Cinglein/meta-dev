use serde::{Deserialize, Serialize};

use crate::budgets::budget_id::BudgetId;
use crate::percent::Percent;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    of: BudgetId,
    percent: Percent,
}

impl Part {
    #[must_use]
    pub fn parent(&self) -> &BudgetId {
        &self.of
    }

    #[must_use]
    pub fn percent(&self) -> Percent {
        self.percent
    }
}
