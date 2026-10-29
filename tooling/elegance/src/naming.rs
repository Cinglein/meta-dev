use std::fmt;

use crate::trait_key::TraitKey;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Naming {
    Trait(TraitKey),
    Composed,
    Missing,
}

impl fmt::Display for Naming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Naming::Trait(key) => write!(formatter, "named by {key}"),
            Naming::Composed => formatter.write_str("named only by a composition of shared traits"),
            Naming::Missing => formatter.write_str("unnamed"),
        }
    }
}
