use std::fmt;

use super::amount::Amount;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Measurement {
    kind: &'static str,
    actual: Amount,
    limit: Amount,
}

impl Measurement {
    pub(super) const fn new(kind: &'static str, actual: Amount, limit: Amount) -> Measurement {
        Measurement { kind, actual, limit }
    }

    pub(super) fn exceeded(&self) -> bool {
        self.actual > self.limit
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "has {} {}, at most {} allowed", self.actual, self.kind, self.limit)
    }
}
