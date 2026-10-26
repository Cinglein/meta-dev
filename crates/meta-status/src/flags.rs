use std::fmt;

use crate::flag::Flag;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Flags(Vec<Flag>);

impl Flags {
    #[must_use]
    pub fn new(flags: Vec<Flag>) -> Self {
        Self(flags)
    }

    #[must_use]
    pub fn raised(&self) -> Option<&Self> {
        self.0.first().map(|_| self)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|flag| writeln!(formatter, "{flag}"))
    }
}
