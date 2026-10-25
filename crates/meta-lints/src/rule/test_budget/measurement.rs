use std::fmt;

use super::amount::Amount;
use crate::site::Site;
use crate::violation::Violation;

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

    pub(super) fn violation(self) -> Option<Violation> {
        self.exceeded().then(|| Violation::weighted(Site::Workspace, self, self.actual.beyond(self.limit)))
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "has {} {}, at most {} allowed", self.actual, self.kind, self.limit)
    }
}
