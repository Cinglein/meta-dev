mod allowance;
mod lines;
mod measure;
mod overspend;
mod source;
mod state;
mod warning_line;
mod window;

use std::collections::BTreeMap;

use meta_budget::{BudgetId, Percent};
use serde::{Deserialize, Serialize};

use super::tree_failure::TreeFailure;
use crate::domains::{Domains, Extent, ScopeKind};
use allowance::Allowance;
pub use allowance::Amount;
pub use lines::Lines;
pub use measure::{BuildStart, ItemFilter, ItemKind, Measure, MutationJob};
use measure::{Enforcer, Unit};
use meta_schema::{Division, TopDivision};
use meta_severity::{Scales, SeverityFilter};
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
    #[serde(default)]
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
            division: Division::from(
                measure
                    .scope()
                    .map_or(TopDivision::FeaturesAndMaintenance, ScopeKind::owner),
            ),
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
    pub fn measure(&self) -> &Measure {
        &self.measure
    }

    #[must_use]
    pub fn standing_in(&self) -> Option<&Division> {
        (self.window == Window::Standing).then_some(&self.division)
    }

    #[must_use]
    pub fn allowance(&self) -> &Allowance {
        &self.allowance
    }

    #[must_use]
    pub fn tolerance(&self) -> Option<Percent> {
        self.tolerance
    }

    #[must_use]
    pub fn blocks(&self, state: Option<State>) -> bool {
        state == Some(State::Over)
            && self.overspend == Overspend::Block
            && (self.standing_in().is_some() || self.measure.enforcer() != Enforcer::Lint)
    }

    #[must_use]
    pub fn state(&self, lines: Option<&Lines>, spend: Option<&Amount>) -> Option<State> {
        match self.measure.unit() {
            Unit::Baseline => spend.map(|_| State::Ok),
            Unit::Duration | Unit::Share | Unit::Count => {
                lines.zip(spend).map(|(lines, spend)| lines.state(spend))
            }
        }
    }

    #[must_use]
    pub fn top_measure(&self) -> Option<Measure> {
        self.allowance.whole().map(|_| self.measure.clone())
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
        self.measure.scope().map_or(Ok(()), |needed| {
            match domains
                .domain(&self.division)
                .map(|domain| domain.extent(needed))
            {
                Some(Extent::Whole) => Ok(()),
                Some(Extent::Partial) => Err(TreeFailure::PartlyMeasurable(id.clone(), needed)),
                Some(Extent::Undeclared) | None => {
                    Err(TreeFailure::Unmeasurable(id.clone(), needed))
                }
            }
        })
    }

    pub fn leveled(&self, id: &BudgetId, scales: &Scales) -> Result<(), TreeFailure> {
        let named = self
            .measure
            .item_filter()
            .and_then(ItemFilter::severity)
            .and_then(SeverityFilter::level);
        named
            .filter(|level| {
                scales
                    .scale(&self.division)
                    .and_then(|scale| scale.level(level))
                    .is_none()
            })
            .map_or(Ok(()), |level| {
                Err(TreeFailure::UnknownLevel(id.clone(), level.clone()))
            })
    }

    pub fn placed(
        &self,
        id: &BudgetId,
        budgets: &BTreeMap<BudgetId, Budget>,
    ) -> Result<(), TreeFailure> {
        let refused = |failure: fn(BudgetId) -> TreeFailure| Err(failure(id.clone()));
        let unit = self.measure.unit();
        match (
            self.allowance.resolved(|_| None),
            self.allowance.parent().map(|parent| budgets.get(parent)),
        ) {
            _ if self
                .measure
                .window()
                .is_some_and(|window| window != self.window) =>
            {
                refused(TreeFailure::OtherWindow)
            }
            _ if self.tolerance.is_some() && self.measure.enforcer() == Enforcer::Lint => {
                refused(TreeFailure::LintTolerance)
            }
            (Some(amount), _) if amount.unit() != unit => refused(TreeFailure::OtherUnit),
            (Some(amount), _) if unit != Unit::Baseline && self.lines(&amount).is_none() => {
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
            (_, Some(Some(parent))) if parent.measure.unit() != unit => {
                refused(TreeFailure::OtherUnit)
            }
            (_, Some(Some(parent)))
                if parent.measure != self.measure
                    && (self.standing_in().is_some() || parent.standing_in().is_some()) =>
            {
                refused(TreeFailure::OtherMeasure)
            }
            (_, None | Some(Some(_))) => Ok(()),
        }
    }
}
