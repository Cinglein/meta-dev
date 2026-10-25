use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Measurement<Count> {
    kind: &'static str,
    actual: Count,
    limit: Count,
}

impl<Count: PartialOrd + fmt::Display> Measurement<Count> {
    pub(super) const fn new(kind: &'static str, actual: Count, limit: Count) -> Measurement<Count> {
        Measurement {
            kind,
            actual,
            limit,
        }
    }

    pub(super) fn overrun(&self) -> Option<String> {
        (self.actual > self.limit).then(|| self.to_string())
    }
}

impl<Count: fmt::Display> fmt::Display for Measurement<Count> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "has {} {}, at most {} allowed",
            self.actual, self.kind, self.limit
        )
    }
}
