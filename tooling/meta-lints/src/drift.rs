use std::fmt;

use crate::violation_count::ViolationCount;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drift {
    rule: String,
    found: ViolationCount,
    allowed: ViolationCount,
}

impl Drift {
    pub fn between(rule: &str, found: ViolationCount, allowed: ViolationCount) -> Option<Drift> {
        (found != allowed).then(|| Drift { rule: rule.to_owned(), found, allowed })
    }

    pub fn grew(&self) -> bool {
        self.found > self.allowed
    }

    pub fn rule(&self) -> &str {
        &self.rule
    }
}

impl fmt::Display for Drift {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {} found, baseline {}", self.rule, self.found, self.allowed)
    }
}
