use std::fmt;

use meta_schema::{Actor, Id, Lease};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLease {
    holder: Actor,
    lease: Id<Lease>,
}

impl LiveLease {
    #[must_use]
    pub fn new(holder: Actor, lease: Id<Lease>) -> Self {
        Self { holder, lease }
    }

    #[must_use]
    pub fn holder(&self) -> &Actor {
        &self.holder
    }

    #[must_use]
    pub fn lease(&self) -> Id<Lease> {
        self.lease
    }
}

impl fmt::Display for LiveLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.holder, self.lease)
    }
}
