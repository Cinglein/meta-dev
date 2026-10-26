use std::fmt;

use crate::site::Site;
use crate::violation_count::ViolationCount;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    site: Site,
    message: String,
    weight: ViolationCount,
}

impl Violation {
    pub fn new(site: Site, message: impl fmt::Display) -> Violation {
        Self::weighted(site, message, ViolationCount::ONE)
    }

    pub fn weighted(site: Site, message: impl fmt::Display, weight: ViolationCount) -> Violation {
        Violation { site, message: message.to_string(), weight }
    }

    pub fn weight(&self) -> ViolationCount {
        self.weight
    }

    pub fn site(&self) -> &Site {
        &self.site
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.site, self.message)
    }
}
