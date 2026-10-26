use std::fmt;

use crate::task::ratchet::violation_count::ViolationCount;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drift {
    lint: String,
    found: ViolationCount,
    allowed: ViolationCount,
}

impl Drift {
    pub fn between(lint: &str, found: ViolationCount, allowed: ViolationCount) -> Option<Drift> {
        (found != allowed).then(|| Drift { lint: lint.to_owned(), found, allowed })
    }

    pub fn grew(&self) -> bool {
        self.found.cmp(&self.allowed).is_gt()
    }

    pub fn lint(&self) -> &str {
        &self.lint
    }
}

impl fmt::Display for Drift {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {} found, baseline {}", self.lint, self.found, self.allowed)
    }
}
