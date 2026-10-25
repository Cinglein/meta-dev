use std::fmt;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct UnnamedSize(Option<Name>);

impl UnnamedSize {
    pub(super) fn new(context: Option<Name>) -> UnnamedSize {
        UnnamedSize(context)
    }
}

impl fmt::Display for UnnamedSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "usize in {} is neither a length nor an index; a count gets a newtype, a size says so in its name",
            self.0.as_ref().map(Name::as_str).unwrap_or_default()
        )
    }
}
