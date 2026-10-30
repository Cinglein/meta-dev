use std::fmt;
use std::str::FromStr;

use meta_budget::BudgetId;
use serde::{Deserialize, Serialize};

use crate::acceptance_failure::AcceptanceFailure;
use crate::point_name::PointName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Enforcement {
    Budget(BudgetId),
    Step(PointName),
    Hook(PointName),
}

impl FromStr for Enforcement {
    type Err = AcceptanceFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let point = match text.split_once(':') {
            Some(("budget", id)) => id.parse().ok().map(Self::Budget),
            Some(("step", name)) => name.parse().ok().map(Self::Step),
            Some(("hook", name)) => name.parse().ok().map(Self::Hook),
            _ => None,
        };
        point.ok_or(AcceptanceFailure::UnreadablePoint)
    }
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
