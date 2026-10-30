use std::fmt;

use serde::{Deserialize, Serialize};

use crate::budget_id::BudgetId;
use crate::hook_name::HookName;
use crate::step_name::StepName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Enforcement {
    Budget(BudgetId),
    Step(StepName),
    Hook(HookName),
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
