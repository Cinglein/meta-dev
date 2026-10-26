use std::fmt;

use crate::crate_summary::CrateSummary;

#[derive(Clone, Debug)]
pub struct GraphReport(Vec<CrateSummary>);

impl GraphReport {
    #[must_use]
    pub fn new(summaries: Vec<CrateSummary>) -> GraphReport {
        GraphReport(summaries)
    }
}

impl fmt::Display for GraphReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|summary| write!(formatter, "{summary}"))
    }
}
