use std::fmt;

use meta_schema::{Division, LevelName, Target};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stranded {
    item: Target,
    division: Division,
    level: LevelName,
}

impl Stranded {
    #[must_use]
    pub fn new(item: Target, division: Division, level: LevelName) -> Stranded {
        Stranded { item, division, level }
    }
}

impl fmt::Display for Stranded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "stranded: {} is {}, not a level of {}'s scale", self.item, self.level, self.division)
    }
}

#[cfg(test)]
mod tests;
