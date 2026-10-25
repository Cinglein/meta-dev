use std::fmt;

use crate::baseline::Baseline;
use crate::count::Count;

#[derive(Clone, Debug)]
pub struct Tally {
    files: Count,
    baseline: Baseline,
}

impl Tally {
    pub fn new(files: Count, baseline: Baseline) -> Tally {
        Tally { files, baseline }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let totals = self.baseline.totals();
        writeln!(formatter, "lint: {} files, {} baselined violations", self.files, totals.values().sum::<Count>())?;
        totals.iter().try_for_each(|(rule, count)| writeln!(formatter, "  {rule}: {count}"))
    }
}
