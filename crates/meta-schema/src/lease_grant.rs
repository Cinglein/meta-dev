use std::fmt;

use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::entity::Lease;
use crate::id::Id;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseGrant {
    id: Id<Lease>,
    issuer: Actor,
}

impl LeaseGrant {
    #[must_use]
    pub fn new(id: Id<Lease>, issuer: Actor) -> Self {
        Self { id, issuer }
    }

    #[must_use]
    pub fn lease(&self) -> Id<Lease> {
        self.id
    }
}

impl fmt::Display for LeaseGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "leased by {}", self.issuer)
    }
}
