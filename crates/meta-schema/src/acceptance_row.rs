use std::fmt;

use meta_amount::Amount;
use serde::{Deserialize, Serialize};

use crate::budget_id::BudgetId;
use crate::measure_name::MeasureName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRow {
    budget: BudgetId,
    measure: MeasureName,
    target: Amount,
}

impl fmt::Display for AcceptanceRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}<={}", self.budget, self.measure, self.target)
    }
}
