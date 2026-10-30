use std::fmt;
use std::str::FromStr;

use meta_budget::{Amount, BudgetId, MeasureName};
use serde::{Deserialize, Serialize};

use crate::acceptance_failure::AcceptanceFailure;

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

impl FromStr for AcceptanceRow {
    type Err = AcceptanceFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let row = text.split_once("<=").and_then(|(named, target)| {
            let (budget, measure) = named.split_once(':')?;
            Some(Self { budget: budget.parse().ok()?, measure: measure.parse().ok()?, target: target.parse().ok()? })
        });
        row.ok_or(AcceptanceFailure::UnreadableRow)
    }
}

impl fmt::Display for AcceptanceRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}<={}", self.budget, self.measure, self.target)
    }
}
