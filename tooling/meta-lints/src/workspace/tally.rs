use std::fmt;

use crate::baseline::Baseline;
use crate::violation_count::ViolationCount;

#[derive(Clone, Debug)]
pub struct Tally {
    files: ViolationCount,
    baseline: Baseline,
}

impl Tally {
    pub fn new(files: ViolationCount, baseline: Baseline) -> Tally {
        Tally { files, baseline }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let totals = self.baseline.totals();
        writeln!(
            formatter,
            "lint: {} files, {} baselined violations",
            self.files,
            totals.values().sum::<ViolationCount>()
        )?;
        totals.iter().try_for_each(|(rule, count)| writeln!(formatter, "  {rule}: {count}"))
    }
}
