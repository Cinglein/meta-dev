use std::fmt;

use syn::Ident;

use super::callee::Callee;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forwarder {
    name: Ident,
    callee: Callee,
}

impl Forwarder {
    pub fn new(name: Ident, callee: Callee) -> Forwarder {
        Forwarder { name, callee }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}

impl fmt::Display for Forwarder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} only forwards to {}; expose the part or call that directly", self.name, self.callee)
    }
}
