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

    #[must_use]
    pub fn issuer(&self) -> &Actor {
        &self.issuer
    }
}

impl fmt::Display for LeaseGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "leased by {}", self.issuer)
    }
}

#[cfg(test)]
mod tests {
    use super::LeaseGrant;
    use crate::id::Id;

    #[test]
    fn a_grant_displays_its_issuer() {
        assert_eq!(LeaseGrant::new(Id::generate(), "owner".parse().unwrap()).to_string(), "leased by owner");
    }
}
