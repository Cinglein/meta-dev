use std::fmt;

use crate::method_name::MethodName;
use crate::trait_key::TraitKey;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodKey {
    implemented: Option<TraitKey>,
    name: MethodName,
}

impl MethodKey {
    #[must_use]
    pub fn new(implemented: Option<TraitKey>, name: MethodName) -> MethodKey {
        MethodKey { implemented, name }
    }

    #[must_use]
    pub fn name(&self) -> &MethodName {
        &self.name
    }
}

impl fmt::Display for MethodKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.implemented {
            Some(implemented) => write!(formatter, "{implemented}::{}", self.name),
            None => write!(formatter, "{}", self.name),
        }
    }
}
