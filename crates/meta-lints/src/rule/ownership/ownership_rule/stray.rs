use std::fmt;

use crate::path_prefix::PathPrefix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stray<'rule> {
    token: &'rule str,
    owner: &'rule PathPrefix,
}

impl<'rule> Stray<'rule> {
    pub fn new(token: &'rule str, owner: &'rule PathPrefix) -> Stray<'rule> {
        Stray { token, owner }
    }
}

impl fmt::Display for Stray<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} outside {}", self.token, self.owner)
    }
}
