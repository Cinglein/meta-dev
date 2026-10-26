use std::fmt;

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::claim::Claim;
use crate::entity::Task;
use crate::entry_note::EntryNote;
use crate::event_visitor::EventVisitor;
use crate::id::Id;
use crate::new_concern::NewConcern;
use crate::new_issue::NewIssue;
use crate::new_task::NewTask;
use crate::schema_version::SchemaVersion;
use crate::task_link::TaskLink;
use crate::task_ref::TaskRef;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Event {
    ConcernIdentified(NewConcern),
    IssueReported(NewIssue),
    TaskOpened(NewTask),
    TaskClaimed(Claim),
    TaskReleased(TaskRef),
    TaskBlocked(Block),
    TaskUnblocked(TaskRef),
    TaskLinked(TaskLink),
    TaskDone(TaskRef),
    TaskReopened(TaskRef),
    EntryRetired(EntryNote),
    EntryCorrected(EntryNote),
    SchemaVersion(SchemaVersion),
}

impl Event {
    #[must_use]
    pub fn claimed_task(&self) -> Option<Id<Task>> {
        match self {
            Self::TaskClaimed(claim) => Some(claim.task()),
            _ => None,
        }
    }

    #[must_use]
    pub fn task_link(&self) -> Option<TaskLink> {
        match self {
            Self::TaskLinked(link) => Some(*link),
            _ => None,
        }
    }

    #[must_use]
    pub fn schema_version(&self) -> Option<SchemaVersion> {
        match self {
            Self::SchemaVersion(version) => Some(*version),
            _ => None,
        }
    }

    pub fn visit<Visitor: EventVisitor>(&self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::ConcernIdentified(concern) => visitor.concern_identified(concern),
            Self::IssueReported(issue) => visitor.issue_reported(issue),
            Self::TaskOpened(task) => visitor.task_opened(task),
            Self::TaskClaimed(claim) => visitor.task_claimed(claim),
            Self::TaskReleased(task) => visitor.task_released(*task),
            Self::TaskBlocked(block) => visitor.task_blocked(*block),
            Self::TaskUnblocked(task) => visitor.task_unblocked(*task),
            Self::TaskLinked(link) => visitor.task_linked(*link),
            Self::TaskDone(task) => visitor.task_done(*task),
            Self::TaskReopened(task) => visitor.task_reopened(*task),
            Self::EntryRetired(note) => visitor.entry_retired(note),
            Self::EntryCorrected(note) => visitor.entry_corrected(note),
            Self::SchemaVersion(version) => visitor.schema_versioned(*version),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConcernIdentified(concern) => write!(formatter, "ConcernIdentified {concern}"),
            Self::IssueReported(issue) => write!(formatter, "IssueReported {issue}"),
            Self::TaskOpened(task) => write!(formatter, "TaskOpened {task}"),
            Self::TaskClaimed(claim) => write!(formatter, "TaskClaimed {claim}"),
            Self::TaskReleased(task) => write!(formatter, "TaskReleased {task}"),
            Self::TaskBlocked(block) => write!(formatter, "TaskBlocked {block}"),
            Self::TaskUnblocked(task) => write!(formatter, "TaskUnblocked {task}"),
            Self::TaskLinked(link) => write!(formatter, "TaskLinked {link}"),
            Self::TaskDone(task) => write!(formatter, "TaskDone {task}"),
            Self::TaskReopened(task) => write!(formatter, "TaskReopened {task}"),
            Self::EntryRetired(note) => write!(formatter, "EntryRetired {note}"),
            Self::EntryCorrected(note) => write!(formatter, "EntryCorrected {note}"),
            Self::SchemaVersion(version) => write!(formatter, "SchemaVersion {version}"),
        }
    }
}
