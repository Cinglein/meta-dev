use std::fmt;

use crate::count::Count;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drift {
    rule: String,
    found: Count,
    allowed: Count,
}

impl Drift {
    pub fn new(rule: &str, found: Count, allowed: Count) -> Drift {
        Drift { rule: rule.to_owned(), found, allowed }
    }

    pub fn moved(&self) -> bool {
        self.found != self.allowed
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
