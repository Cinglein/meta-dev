use std::fmt;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TextOutsideDoor(Name);

impl TextOutsideDoor {
    pub(super) fn new(handler: Name) -> TextOutsideDoor {
        TextOutsideDoor(handler)
    }
}

impl fmt::Display for TextOutsideDoor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} handles text outside a door; parse in FromStr or render in Display", self.0)
    }
}
