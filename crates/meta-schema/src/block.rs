use std::fmt;

use serde::{Deserialize, Serialize};

use crate::blocker::Blocker;
use crate::commit_id::CommitId;
use crate::due_time::DueTime;
use crate::entity::Task;
use crate::id::Id;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    task: Id<Task>,
    on: Blocker,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    head: Option<CommitId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reply_due: Option<DueTime>,
}

impl Block {
    #[must_use]
    pub fn new(task: Id<Task>, on: Blocker) -> Self {
        Self { task, on, head: None, reply_due: None }
    }

    #[must_use]
    pub fn at_head(self, head: CommitId) -> Self {
        Self { head: Some(head), ..self }
    }

    #[must_use]
    pub fn reply_by(self, due: DueTime) -> Self {
        Self { reply_due: Some(due), ..self }
    }

    #[must_use]
    pub fn reply_due(&self) -> Option<DueTime> {
        self.reply_due
    }

    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }

    #[must_use]
    pub fn blocker(&self) -> Blocker {
        self.on
    }

    #[must_use]
    pub fn head(&self) -> Option<&CommitId> {
        self.head.as_ref()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", self.task, self.on)?;
        self.head.as_ref().map_or(Ok(()), |head| write!(formatter, " at {head}"))?;
        self.reply_due.map_or(Ok(()), |due| write!(formatter, " reply due {due}"))
    }
}
