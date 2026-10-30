use std::fmt;

use meta_schema::{Actor, LeaseGrant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLease {
    holder: Actor,
    grant: LeaseGrant,
}

impl LiveLease {
    #[must_use]
    pub fn new(holder: Actor, grant: LeaseGrant) -> Self {
        Self { holder, grant }
    }

    #[must_use]
    pub fn holder(&self) -> &Actor {
        &self.holder
    }

    #[must_use]
    pub fn grant(&self) -> &LeaseGrant {
        &self.grant
    }
}

impl fmt::Display for LiveLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} leased", self.holder)
    }
}
