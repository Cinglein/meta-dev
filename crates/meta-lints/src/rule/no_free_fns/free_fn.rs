use std::fmt;

use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreeFn {
    name: Name,
}

impl FreeFn {
    pub fn new(name: &Ident) -> FreeFn {
        FreeFn { name: Name::from(name) }
    }
}

impl fmt::Display for FreeFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {}", self.name)
    }
}
