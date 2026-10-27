use std::fmt;
use std::iter::Sum;
use std::ops::Add;

use serde::{Deserialize, Serialize};
use task::Report;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViolationCount(usize);

impl ViolationCount {
    pub fn in_report(report: &Report) -> ViolationCount {
        ViolationCount(report.violations().len())
    }
}

impl Add for ViolationCount {
    type Output = ViolationCount;

    fn add(self, other: ViolationCount) -> ViolationCount {
        ViolationCount(self.0 + other.0)
    }
}

impl<'count> Sum<&'count ViolationCount> for ViolationCount {
    fn sum<Counts: Iterator<Item = &'count ViolationCount>>(counts: Counts) -> ViolationCount {
        counts.copied().fold(ViolationCount::default(), Add::add)
    }
}

impl fmt::Display for ViolationCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
