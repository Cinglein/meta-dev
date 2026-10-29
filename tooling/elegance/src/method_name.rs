use std::fmt;

use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodName(String);

impl MethodName {
    #[must_use]
    pub fn new(ident: &Ident) -> MethodName {
        MethodName(ident.to_string())
    }
}

impl fmt::Display for MethodName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
