use std::fmt;

use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodName(Ident);

impl MethodName {
    #[must_use]
    pub fn new(ident: &Ident) -> MethodName {
        MethodName(ident.clone())
    }
}

impl fmt::Display for MethodName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
