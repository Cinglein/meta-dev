mod allowance;
mod lines;
mod measure;
mod overspend;
mod source;
mod state;
mod warning_line;
mod window;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::budget_id::BudgetId;
use super::tree_failure::TreeFailure;
use crate::domains::{Domains, Extent};
use crate::percent::Percent;
use allowance::Allowance;
pub use allowance::Amount;
pub use lines::Lines;
use measure::{Enforcer, Unit};
pub use measure::{Measure, MutationJob};
use meta_schema::Division;
use overspend::Overspend;
use source::Source;
pub use state::State;
use warning_line::WarningLine;
use window::Window;

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
    pub fn legacy(measure: Measure, amount: Amount, tolerance: Option<Percent>) -> Budget {
        Budget {
            division: Division::from(measure.scope().owner()),
            measure,
            allowance: Allowance::Whole(amount),
            window: Window::PerChange,
            source: Source::Ci,
            warning: WarningLine::default(),
            tolerance,
            overspend: Overspend::default(),
        }
    }

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
    pub fn tolerance(&self) -> Option<Percent> {
        self.tolerance
    }

    #[must_use]
    pub fn blocks(&self, state: Option<State>) -> bool {
        state == Some(State::Over) && self.overspend == Overspend::Block
    }

    #[must_use]
    pub fn top_measure(&self) -> Option<Measure> {
        self.allowance.whole().map(|_| self.measure)
    }

    #[must_use]
    pub fn blocking_line(&self) -> Option<Amount> {
        self.allowance
            .whole()
            .and_then(|allowance| self.lines(allowance))
            .map(|lines| lines.blocking().clone())
    }

    #[must_use]
    pub fn lines(&self, allowance: &Amount) -> Option<Lines> {
        let blocking = Percent::total([Percent::ALL, self.tolerance.unwrap_or(Percent::NONE)])?;
        Some(Lines::new(
            allowance.part(self.warning.percent())?,
            allowance.part(blocking)?,
        ))
    }

    pub fn measurable(&self, id: &BudgetId, domains: &Domains) -> Result<(), TreeFailure> {
        let needed = self.measure.scope();
        match domains
            .domain(&self.division)
            .map(|domain| domain.extent(needed))
        {
            Some(Extent::Whole) => Ok(()),
            Some(Extent::Partial) => Err(TreeFailure::PartlyMeasurable(id.clone(), needed)),
            Some(Extent::Undeclared) | None => Err(TreeFailure::Unmeasurable(id.clone(), needed)),
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
            _ if self.tolerance.is_some() && self.measure.enforcer() == Enforcer::Lint => {
                refused(TreeFailure::LintTolerance)
            }
            (Some(amount), _) if amount.unit() != unit => refused(TreeFailure::OtherUnit),
            (Some(amount), _) if unit != Unit::Baseline && self.lines(amount).is_none() => {
                refused(TreeFailure::Overflow)
            }
            (_, None) if self.division.is_nested() => refused(TreeFailure::NestedWhole),
            (_, Some(None)) => refused(TreeFailure::NoParent),
            (_, Some(Some(parent))) if parent.measure.unit() == Unit::Baseline => {
                refused(TreeFailure::BaselinePart)
            }
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
