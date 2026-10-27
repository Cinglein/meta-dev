use std::fmt;

use meta_schema::{Division, LevelName, Target};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Demotion {
    Lowered { item: Target, division: Division, from: LevelName, to: LevelName },
    Incomparable { item: Target, division: Division },
}

impl fmt::Display for Demotion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lowered { item, division, from, to } => {
                write!(
                    formatter,
                    "lowered by policy: {item} at {division}: {from} became {to}; lower it with meta rate"
                )
            }
            Self::Incomparable { item, division } => {
                write!(formatter, "not comparable: {item} at {division}: the scale's levels changed")
            }
        }
    }
}

#[cfg(test)]
mod tests;
