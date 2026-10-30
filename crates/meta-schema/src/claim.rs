use std::fmt;

use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::entity::{Lease, Task};
use crate::id::Id;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    task: Id<Task>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    from: Option<Actor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lease: Option<Id<Lease>>,
}

impl Claim {
    #[must_use]
    pub fn new(task: Id<Task>, from: Option<Actor>) -> Self {
        Self { task, from, lease: None }
    }

    #[must_use]
    pub fn leased(self) -> Self {
        Self { lease: Some(Id::generate()), ..self }
    }

    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }

    #[must_use]
    pub fn previous_claimant(&self) -> Option<&Actor> {
        self.from.as_ref()
    }

    #[must_use]
    pub fn lease(&self) -> Option<Id<Lease>> {
        self.lease
    }
}

impl fmt::Display for Claim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.task)?;
        self.from.as_ref().map_or(Ok(()), |from| write!(formatter, " from {from}"))?;
        self.lease.map_or(Ok(()), |lease| write!(formatter, " {lease}"))
    }
}
