use std::fmt;

use super::row::Row;
use crate::report::Report;
use crate::selection::Selection;
use crate::site::Site;
use crate::violation::Violation;

#[derive(Debug)]
pub struct Statement<'policy>(Vec<Row<'policy>>);

impl<'policy> Statement<'policy> {
    #[must_use]
    pub fn new(rows: Vec<Row<'policy>>) -> Statement<'policy> {
        Statement(rows)
    }

    #[must_use]
    pub fn verdict(&self, selection: &Selection) -> Report {
        Report::new(
            "budget over its blocking line",
            self.0
                .iter()
                .filter(|row| selection.reaching().is_some() && row.blocks())
                .map(|row| Violation::new(Site::Workspace, row))
                .collect(),
        )
    }
}

impl fmt::Display for Statement<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Row::HEADER)?;
        self.0
            .iter()
            .try_for_each(|row| writeln!(formatter, "{row}"))
    }
}
