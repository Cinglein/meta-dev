use std::collections::BTreeSet;
use std::fmt;

use crate::finding::Finding;
use crate::located::Located;
use crate::places::Places;
use crate::source_path::SourcePath;
use crate::tally::Tally;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Findings<Subject, Detail>(Vec<Finding<Subject, Detail>>);

impl<Subject, Detail> Findings<Subject, Detail> {
    #[must_use]
    pub fn new(findings: impl IntoIterator<Item = Finding<Subject, Detail>>) -> Findings<Subject, Detail> {
        Findings(findings.into_iter().collect())
    }

    #[must_use]
    pub fn sorted_by(
        mut self,
        order: impl FnMut(&Finding<Subject, Detail>, &Finding<Subject, Detail>) -> std::cmp::Ordering,
    ) -> Findings<Subject, Detail> {
        self.0.sort_by(order);
        self
    }

    #[must_use]
    pub fn found(&self) -> &[Finding<Subject, Detail>] {
        &self.0
    }
}

impl<Subject: Located, Detail: Located> Findings<Subject, Detail> {
    #[must_use]
    pub fn located(&self, places: &Places) -> Vec<BTreeSet<SourcePath>> {
        self.0.iter().map(|finding| finding.places(places)).collect()
    }
}

impl<Subject: fmt::Display, Detail: fmt::Display> fmt::Display for Findings<Subject, Detail> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "  found {}", self.0.len())?;
        self.0.iter().take(Tally::SHOWN).try_for_each(|finding| writeln!(formatter, "    {finding}"))
    }
}
