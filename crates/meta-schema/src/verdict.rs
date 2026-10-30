use std::fmt;

use serde::{Deserialize, Serialize};

use crate::budget_id::BudgetId;
use crate::enforcement::Enforcement;
use crate::gap::Gap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Instrumented,
    Uninstrumented(Gap),
    OnTarget,
    OffTarget(BudgetId),
    NotGating(Enforcement),
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instrumented => formatter.write_str("instrumented"),
            Self::Uninstrumented(gap) => write!(formatter, "uninstrumented {gap}"),
            Self::OnTarget => formatter.write_str("on target"),
            Self::OffTarget(budget) => write!(formatter, "off target {budget}"),
            Self::NotGating(point) => write!(formatter, "not gating {point}"),
        }
    }
}
