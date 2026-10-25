use std::fmt;

use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoolPredicate {
    name: Name,
}

impl BoolPredicate {
    pub fn new(name: &Ident) -> BoolPredicate {
        BoolPredicate { name: Name::from(name) }
    }
}

impl fmt::Display for BoolPredicate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} -> bool", self.name)
    }
}
