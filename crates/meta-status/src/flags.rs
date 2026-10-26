use std::fmt;

use serde::{Deserialize, Serialize};

use crate::flag::Flag;
use crate::judgement::Judgement;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
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

    #[must_use]
    pub fn since(&self, previous: &Self) -> Self {
        Self(self.0.iter().filter(|flag| !previous.0.contains(flag)).cloned().collect())
    }

    #[must_use]
    pub fn awaiting_judgement(&self) -> Self {
        Self(self.0.iter().filter(|flag| flag.judgement() == Judgement::Needed).cloned().collect())
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|flag| writeln!(formatter, "{flag}"))
    }
}

#[cfg(test)]
mod tests;
