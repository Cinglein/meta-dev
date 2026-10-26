use std::fmt;

use super::super::budget::{Amount, Budget, Lines, State};
use super::super::budget_id::BudgetId;

#[derive(Debug)]
pub struct Row<'policy> {
    id: &'policy BudgetId,
    budget: &'policy Budget,
    lines: Option<Lines>,
    spend: Option<Amount>,
}

impl<'policy> Row<'policy> {
    pub const HEADER: &str = "budget             division                   measure                     source          spend    warning   blocking state";
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
        }
    }

    #[must_use]
    pub fn state(&self) -> Option<State> {
        self.lines
            .as_ref()
            .zip(self.spend.as_ref())
            .map(|(lines, spend)| lines.state(spend))
    }

    #[must_use]
    pub fn blocks(&self) -> bool {
        self.budget.blocks(self.state())
    }
}

impl fmt::Display for Row<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = |value: Option<String>| value.unwrap_or_else(|| Self::UNMEASURED.to_owned());
        write!(
            formatter,
            "{:<18} {:<26} {:<27} {:<10} {:>10} {:>21} {}",
            self.id.to_string(),
            self.budget.division().to_string(),
            self.budget.measure().to_string(),
            self.budget.source().to_string(),
            shown(self.spend.as_ref().map(ToString::to_string)),
            shown(self.lines.as_ref().map(ToString::to_string)),
            shown(self.state().map(|state| state.to_string())),
        )
    }
}
