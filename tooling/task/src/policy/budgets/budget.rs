mod allowance;
mod measure;
mod overspend;
mod source;
mod warning_line;
mod window;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::budget_id::BudgetId;
use super::tree_failure::TreeFailure;
use crate::policy::domains::Domains;
use crate::policy::percent::Percent;
pub use allowance::Allowance;
pub use measure::Measure;
use meta_schema::Division;
pub use overspend::Overspend;
pub use source::Source;
pub use warning_line::WarningLine;
pub use window::Window;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    division: Division,
    measure: Measure,
    allowance: Allowance,
    window: Window,
    source: Source,
    #[serde(default)]
    warning: WarningLine,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tolerance: Option<Percent>,
    #[serde(default)]
    overspend: Overspend,
}

impl Budget {
    #[must_use]
    pub fn division(&self) -> &Division {
        &self.division
    }

    #[must_use]
    pub fn measure(&self) -> Measure {
        self.measure
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
    pub fn source(&self) -> Source {
        self.source
    }

    #[must_use]
    pub fn warning(&self) -> WarningLine {
        self.warning
    }

    #[must_use]
    pub fn tolerance(&self) -> Option<Percent> {
        self.tolerance
    }

    #[must_use]
    pub fn overspend(&self) -> Overspend {
        self.overspend
    }

    pub fn measurable(&self, id: &BudgetId, domains: &Domains) -> Result<(), TreeFailure> {
        let needed = self.measure.scope();
        match domains.domain(&self.division) {
            Some(domain) if domain.declares(needed) => Ok(()),
            Some(_) | None => Err(TreeFailure::Unmeasurable(id.clone(), needed)),
        }
    }

    pub fn placed(
        &self,
        id: &BudgetId,
        budgets: &BTreeMap<BudgetId, Budget>,
    ) -> Result<(), TreeFailure> {
        let refused = |failure: fn(BudgetId) -> TreeFailure| Err(failure(id.clone()));
        let unit = self.measure.unit();
        match (
            self.allowance.whole(),
            self.allowance.part().map(|part| budgets.get(part.parent())),
        ) {
            (Some(amount), _) if amount.unit() != unit => refused(TreeFailure::OtherUnit),
            (_, None) if self.division.is_nested() => refused(TreeFailure::NestedWhole),
            (_, Some(None)) => refused(TreeFailure::NoParent),
            (_, Some(Some(parent))) if !parent.division.strictly_contains(&self.division) => {
                refused(TreeFailure::NotNested)
            }
            (_, Some(Some(parent))) if parent.window != self.window => {
                refused(TreeFailure::OtherWindow)
            }
            (_, Some(Some(parent))) if parent.measure.unit() != unit => {
                refused(TreeFailure::OtherUnit)
            }
            (_, None | Some(Some(_))) => Ok(()),
        }
    }
}
