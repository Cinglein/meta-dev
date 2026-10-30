use std::fmt;

use meta_budget::BudgetId;
use serde::{Deserialize, Serialize};

use crate::point_name::PointName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Enforcement {
    Budget(BudgetId),
    Step(PointName),
    Hook(PointName),
}

impl fmt::Display for Enforcement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget(budget) => write!(formatter, "budget:{budget}"),
            Self::Step(step) => write!(formatter, "step:{step}"),
            Self::Hook(hook) => write!(formatter, "hook:{hook}"),
        }
    }
}
