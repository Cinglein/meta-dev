use std::fmt;

use crate::count::Count;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drift {
    rule: String,
    file: String,
    found: Count,
    allowed: Count,
}

impl Drift {
    pub fn new(rule: &str, file: &str, found: Count, allowed: Count) -> Drift {
        Drift { rule: rule.to_owned(), file: file.to_owned(), found, allowed }
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

    pub fn file(&self) -> &str {
        &self.file
    }
}

impl fmt::Display for Drift {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: {} found, baseline {}", self.rule, self.file, self.found, self.allowed)
    }
}
