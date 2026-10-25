use std::fmt;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BareNumber {
    primitive: Name,
    context: Option<Name>,
}

impl BareNumber {
    pub(super) fn new(primitive: Name, context: Option<Name>) -> BareNumber {
        BareNumber { primitive, context }
    }
}

impl fmt::Display for BareNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "bare {} in {}; name what it counts with a newtype",
            self.primitive,
            self.context.as_ref().map(Name::as_str).unwrap_or_default()
        )
    }
}
