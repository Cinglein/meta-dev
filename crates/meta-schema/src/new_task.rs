use std::fmt;

use serde::{Deserialize, Serialize};

use crate::division::Division;
use crate::due_time::DueTime;
use crate::entity::Task;
use crate::id::Id;
use crate::placement::Placement;
use crate::target::Target;
use crate::text::Text;
use crate::tokens::Tokens;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTask {
    task: Id<Task>,
    text: Text,
    #[serde(default, skip_serializing_if = "Placement::is_root")]
    division: Placement,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    addresses: Vec<Target>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effort: Option<Tokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    due: Option<DueTime>,
}

impl NewTask {
    #[must_use]
    pub fn new(division: Division, text: Text, addresses: Vec<Target>) -> Self {
        Self { task: Id::generate(), text, division: Placement::new(division), addresses, effort: None, due: None }
    }

    #[must_use]
    pub fn allowing(self, effort: Tokens) -> Self {
        Self { effort: Some(effort), ..self }
    }

    #[must_use]
    pub fn due_by(self, due: DueTime) -> Self {
        Self { due: Some(due), ..self }
    }

    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.division
    }

    #[must_use]
    pub fn addresses(&self) -> &[Target] {
        &self.addresses
    }

    #[must_use]
    pub fn effort_allowance(&self) -> Option<Tokens> {
        self.effort
    }

    #[must_use]
    pub fn due_time(&self) -> Option<DueTime> {
        self.due
    }

    #[must_use]
    pub fn corrected(&self, text: Text) -> Self {
        Self { text, ..self.clone() }
    }
}

impl fmt::Display for NewTask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}{}", self.task, self.text, self.division)?;
        self.addresses.iter().try_for_each(|target| write!(formatter, " >{target}"))?;
        self.effort.map_or(Ok(()), |effort| write!(formatter, " effort {effort}"))?;
        self.due.map_or(Ok(()), |due| write!(formatter, " due {due}"))
    }
}
