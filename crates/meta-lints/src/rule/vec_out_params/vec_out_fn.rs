use std::fmt;

use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VecOutFn {
    name: Name,
}

impl VecOutFn {
    pub fn new(name: &Ident) -> VecOutFn {
        VecOutFn { name: Name::from(name) }
    }
}

impl fmt::Display for VecOutFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {}", self.name)
    }
}
