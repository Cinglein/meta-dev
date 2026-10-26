use std::fmt;

use crate::task::ratchet::drift::Drift;

#[derive(Debug)]
pub struct Raised(Vec<Drift>);

impl Raised {
    pub fn new(drifts: Vec<Drift>) -> Raised {
        Raised(drifts)
    }
}

impl fmt::Display for Raised {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "baseline raised above the merge base:")?;
        self.0.iter().try_for_each(|drift| write!(formatter, "\n{drift}"))
    }
}
