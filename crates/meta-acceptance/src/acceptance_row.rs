use std::fmt;

use meta_budget::{Amount, BudgetId, MeasureName};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRow {
    budget: BudgetId,
    measure: MeasureName,
    target: Amount,
}

impl AcceptanceRow {
    #[must_use]
    pub fn budget(&self) -> &BudgetId {
        &self.budget
    }
}

impl fmt::Display for AcceptanceRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}<={}", self.budget, self.measure, self.target)
    }
}
