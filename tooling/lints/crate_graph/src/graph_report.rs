use std::fmt;

use task::{Amount, Most, Percent, Policy, Report};

use crate::crate_summary::CrateSummary;

#[derive(Clone, Debug, Default)]
pub struct GraphReport(Vec<CrateSummary>);

impl GraphReport {
    pub(crate) fn new(summaries: Vec<CrateSummary>) -> GraphReport {
        GraphReport(summaries)
    }

    #[must_use]
    pub fn report(&self, policy: &Policy) -> Report {
        Report::new(
            "a crate whose items form one big cycle or two loosely linked groups is split",
            self.0
                .iter()
                .flat_map(|summary| summary.violations(policy))
                .collect(),
        )
    }

    #[must_use]
    pub fn largest_tangle(&self) -> Amount {
        let largest = self
            .0
            .iter()
            .map(CrateSummary::largest_tangle)
            .max()
            .unwrap_or_default();
        Amount::Count(Most::from(largest.count()))
    }

    #[must_use]
    pub fn effect_share(&self) -> Percent {
        self.0
            .iter()
            .map(|summary| summary.effects().share())
            .max()
            .unwrap_or(Percent::NONE)
    }

    #[must_use]
    pub fn effect_dispersion(&self) -> Most {
        let widest = self
            .0
            .iter()
            .map(|summary| summary.effects().dispersion())
            .max()
            .unwrap_or_default();
        Most::from(widest.count())
    }
}

impl fmt::Display for GraphReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|summary| write!(formatter, "{summary}"))
    }
}
