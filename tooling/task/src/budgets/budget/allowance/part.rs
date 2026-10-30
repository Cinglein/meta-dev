use meta_budget::{BudgetId, Percent};
use serde::{Deserialize, Serialize};

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
