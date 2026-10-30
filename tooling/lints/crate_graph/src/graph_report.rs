use std::fmt;

use task::{Amount, Most, PackageGraph, Percent, Policy, Report};

use crate::crate_summary::CrateSummary;

#[derive(Clone, Debug, Default)]
pub struct GraphReport {
    summaries: Vec<CrateSummary>,
    packages: PackageGraph,
}

impl GraphReport {
    pub(crate) fn new(summaries: Vec<CrateSummary>, packages: PackageGraph) -> GraphReport {
        GraphReport {
            summaries,
            packages,
        }
    }

    #[must_use]
    pub fn packages(&self) -> &PackageGraph {
        &self.packages
    }

    #[must_use]
    pub fn report(&self, policy: &Policy) -> Report {
        Report::new(
            "a crate whose items form one big cycle or two loosely linked groups is split",
            self.summaries
                .iter()
                .flat_map(|summary| summary.violations(policy))
                .collect(),
        )
    }

    #[must_use]
    pub fn largest_tangle(&self) -> Amount {
        let largest = self
            .summaries
            .iter()
            .map(CrateSummary::largest_tangle)
            .max()
            .unwrap_or_default();
        Amount::Count(Most::from(largest.count()))
    }

    #[must_use]
    pub fn effect_share(&self) -> Percent {
        self.summaries
            .iter()
            .map(|summary| summary.effects().share())
            .max()
            .unwrap_or(Percent::NONE)
    }

    #[must_use]
    pub fn effect_dispersion(&self) -> Most {
        let widest = self
            .summaries
            .iter()
            .map(|summary| summary.effects().dispersion())
            .max()
            .unwrap_or_default();
        Most::from(widest.count())
    }
}

impl fmt::Display for GraphReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.summaries
            .iter()
            .try_for_each(|summary| write!(formatter, "{summary}"))
    }
}
