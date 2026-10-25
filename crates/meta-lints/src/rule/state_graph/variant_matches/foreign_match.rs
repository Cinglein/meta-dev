use std::fmt;

use crate::rule::name::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForeignMatch<'name>(&'name Name);

impl<'name> ForeignMatch<'name> {
    pub fn new(enum_name: &'name Name) -> ForeignMatch<'name> {
        ForeignMatch(enum_name)
    }
}

impl fmt::Display for ForeignMatch<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "match on {} outside its file; dispatch through a trait", self.0)
    }
}
