use std::fmt;

use crate::count::Count;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReexportLimit(usize);

impl ReexportLimit {
    pub const MAX: ReexportLimit = ReexportLimit(8);

    pub fn exceeded_by(self, count: Count) -> bool {
        count > Count::of_len(self.0)
    }
}

impl fmt::Display for ReexportLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
