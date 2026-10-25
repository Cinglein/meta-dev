use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Task;
use crate::id::Id;
use crate::target::Target;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTask {
    task: Id<Task>,
    text: Text,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    addresses: Vec<Target>,
}

impl NewTask {
    #[must_use]
    pub fn new(text: Text, addresses: Vec<Target>) -> Self {
        Self { task: Id::generate(), text, addresses }
    }

    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }

    #[must_use]
    pub fn addresses(&self) -> &[Target] {
        &self.addresses
    }

    #[must_use]
    pub fn corrected(&self, text: Text) -> Self {
        Self { text, ..self.clone() }
    }
}

impl fmt::Display for NewTask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.task, self.text)?;
        self.addresses.iter().try_for_each(|target| write!(formatter, " >{target}"))
    }
}
