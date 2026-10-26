mod allowance;
mod division;
mod overspend;
mod window;

use std::collections::BTreeMap;

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

    pub fn placed(
        &self,
        id: &BudgetId,
        budgets: &BTreeMap<BudgetId, Budget>,
    ) -> Result<(), TreeFailure> {
        let refused = |failure: fn(BudgetId) -> TreeFailure| Err(failure(id.clone()));
        match self.allowance.part().map(|part| budgets.get(part.parent())) {
            None if self.division.is_nested() => refused(TreeFailure::NestedWhole),
            Some(None) => refused(TreeFailure::NoParent),
            Some(Some(parent)) if !parent.division.strictly_contains(&self.division) => {
                refused(TreeFailure::NotNested)
            }
            Some(Some(parent)) if parent.window != self.window => refused(TreeFailure::OtherWindow),
            None | Some(Some(_)) => Ok(()),
        }
    }
}
