use std::fmt;

use crate::request::project::TickLimit;

#[derive(Debug)]
pub struct GaveUp(TickLimit);

impl GaveUp {
    pub fn after(limit: TickLimit) -> Self {
        Self(limit)
    }
}

impl fmt::Display for GaveUp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "tick gave up after {}", self.0)
    }
}
