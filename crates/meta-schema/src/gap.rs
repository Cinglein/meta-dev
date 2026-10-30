use std::fmt;

use meta_acceptance::SpanName;
use meta_budget::BudgetId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gap {
    Unmeasured(BudgetId),
    NoSpan(SpanName),
}

impl fmt::Display for Gap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unmeasured(budget) => write!(formatter, "unmeasured {budget}"),
            Self::NoSpan(span) => write!(formatter, "no span {span}"),
        }
    }
}
