use std::fmt;

use crate::count::Count;
use crate::site::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    site: Site,
    message: String,
    weight: Count,
}

impl Violation {
    pub fn new(site: Site, message: impl fmt::Display) -> Violation {
        Self::weighted(site, message, Count::ONE)
    }

    pub fn weighted(site: Site, message: impl fmt::Display, weight: Count) -> Violation {
        Violation { site, message: message.to_string(), weight }
    }

    pub fn weight(&self) -> Count {
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
