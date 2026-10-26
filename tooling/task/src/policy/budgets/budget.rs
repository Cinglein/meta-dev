mod allowance;
mod division;
mod overspend;
mod window;

use serde::{Deserialize, Serialize};

use super::budget_id::BudgetId;
use super::tree_failure::TreeFailure;
pub use allowance::Allowance;
use division::Division;
pub use overspend::Overspend;
pub use window::Window;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    division: Division,
    allowance: Allowance,
    window: Window,
    #[serde(default)]
    overspend: Overspend,
}

impl Budget {
    #[must_use]
    pub fn division(&self) -> &Division {
        &self.division
    }

    #[must_use]
    pub fn allowance(&self) -> &Allowance {
        &self.allowance
    }

    #[must_use]
    pub fn window(&self) -> Window {
        self.window
    }

    #[must_use]
    pub fn overspend(&self) -> Overspend {
        self.overspend
    }

    pub fn placed_under(&self, id: &BudgetId, parent: Option<&Budget>) -> Result<(), TreeFailure> {
        match parent {
            None => Err(TreeFailure::NoParent(id.clone())),
            Some(parent) if parent.division.strictly_contains(&self.division) => Ok(()),
            Some(_) => Err(TreeFailure::NotNested(id.clone())),
        }
    }
}
