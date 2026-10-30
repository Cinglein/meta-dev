use std::fmt;

use crate::reading::Reading;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promoted(Reading);

impl Promoted {
    pub(crate) fn new(reading: Reading) -> Promoted {
        Promoted(reading)
    }

    #[must_use]
    pub fn reading(&self) -> &Reading {
        &self.0
    }
}

impl fmt::Display for Promoted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
