use std::fmt;

use crate::task::ratchet::Drift;

#[derive(Debug)]
pub struct Unmatched(Vec<Drift>);

impl Unmatched {
    pub fn new(drifts: Vec<Drift>) -> Unmatched {
        Unmatched(drifts)
    }
}

impl fmt::Display for Unmatched {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "process counts differ from their allowances; set each row to its count:")?;
        self.0.iter().try_for_each(|drift| write!(formatter, "\n{drift}"))
    }
}
