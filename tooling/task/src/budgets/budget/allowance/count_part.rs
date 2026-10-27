use serde::{Deserialize, Serialize};

use crate::budgets::budget_id::BudgetId;
use crate::policy::Most;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountPart {
    of: BudgetId,
    count: Most,
}

impl CountPart {
    #[must_use]
    pub fn parent(&self) -> &BudgetId {
        &self.of
    }

    #[must_use]
    pub fn count(&self) -> Most {
        self.count
    }
}
