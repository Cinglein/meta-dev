use std::fmt;

use crate::rule::name::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivateFnLimit(usize);

impl PrivateFnLimit {
    pub const fn new(most: usize) -> PrivateFnLimit {
        PrivateFnLimit(most)
    }

    pub fn exceeded_by(self, names: &[Name]) -> bool {
        names.len() > self.0
    }
}

impl fmt::Display for PrivateFnLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
