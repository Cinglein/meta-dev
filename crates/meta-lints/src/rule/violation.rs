use std::fmt;

use crate::site::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    site: Site,
    message: String,
}

impl Violation {
    pub fn new(site: Site, message: impl fmt::Display) -> Violation {
        Violation { site, message: message.to_string() }
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
