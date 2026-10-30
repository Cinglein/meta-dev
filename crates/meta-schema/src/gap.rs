use std::fmt;

use meta_trace::SpanKind;
use serde::{Deserialize, Serialize};

use crate::budget_id::BudgetId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gap {
    Unmeasured(BudgetId),
    NoSpan(SpanKind),
}

impl fmt::Display for Gap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unmeasured(budget) => write!(formatter, "unmeasured {budget}"),
            Self::NoSpan(span) => write!(formatter, "no span {span}"),
        }
    }
}
