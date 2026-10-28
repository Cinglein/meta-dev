use std::fmt;

use super::super::budget::{Amount, Budget, Lines, State};
use super::super::budget_id::BudgetId;
use super::super::standing::Exposure;
use crate::selection::Selection;

#[derive(Debug)]
pub struct Row<'policy> {
    id: &'policy BudgetId,
    budget: &'policy Budget,
    lines: Option<Lines>,
    spend: Option<Amount>,
    exposure: Exposure,
}

impl<'policy> Row<'policy> {
    pub const HEADER: &str = "budget             division                   measure                          spend    warning   blocking state";
    const UNMEASURED: &str = "-";

    #[must_use]
    pub fn new(
        id: &'policy BudgetId,
        budget: &'policy Budget,
        lines: Option<Lines>,
        spend: Option<Amount>,
    ) -> Row<'policy> {
        Row {
            id,
            budget,
            lines,
            spend,
            exposure: Exposure::Diff,
        }
    }

    #[must_use]
    pub fn within(self, exposure: Exposure) -> Row<'policy> {
        Row { exposure, ..self }
    }

    #[must_use]
    pub fn state(&self) -> Option<State> {
        self.budget.state(self.lines.as_ref(), self.spend.as_ref())
    }

    #[must_use]
    pub fn blocks(&self, selection: &Selection) -> bool {
        self.exposure.exposed(selection).is_some() && self.budget.blocks(self.state())
    }
}

impl fmt::Display for Row<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = |value: Option<String>| value.unwrap_or_else(|| Self::UNMEASURED.to_owned());
        write!(
            formatter,
            "{:<18} {:<26} {:<27} {:>10} {:>21} {}",
            self.id.to_string(),
            self.budget.division().to_string(),
            self.budget.measure().to_string(),
            shown(
                self.spend
                    .as_ref()
                    .filter(|spend| spend.baseline().is_none())
                    .map(ToString::to_string)
            ),
            shown(self.lines.as_ref().map(ToString::to_string)),
            shown(self.state().map(|state| state.to_string())),
        )?;
        [self.exposure.excuse(), self.budget.excuse()]
            .into_iter()
            .flatten()
            .filter(|_| self.state() == Some(State::Over))
            .try_for_each(|excuse| write!(formatter, ", {excuse}"))
    }
}
