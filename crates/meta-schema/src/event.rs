use std::fmt;

use serde::{Deserialize, Serialize};

use crate::claim::Claim;
use crate::event_visitor::EventVisitor;
use crate::new_concern::NewConcern;
use crate::new_issue::NewIssue;
use crate::new_task::NewTask;
use crate::task_ref::TaskRef;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Event {
    ConcernIdentified(NewConcern),
    IssueReported(NewIssue),
    TaskOpened(NewTask),
    TaskClaimed(Claim),
    TaskReleased(TaskRef),
    TaskDone(TaskRef),
    TaskReopened(TaskRef),
}

impl Event {
    pub fn visit<Visitor: EventVisitor>(&self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::ConcernIdentified(concern) => visitor.concern_identified(concern),
            Self::IssueReported(issue) => visitor.issue_reported(issue),
            Self::TaskOpened(task) => visitor.task_opened(task),
            Self::TaskClaimed(claim) => visitor.task_claimed(claim),
            Self::TaskReleased(task) => visitor.task_released(*task),
            Self::TaskDone(task) => visitor.task_done(*task),
            Self::TaskReopened(task) => visitor.task_reopened(*task),
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
            Self::TaskDone(task) => write!(formatter, "TaskDone {task}"),
            Self::TaskReopened(task) => write!(formatter, "TaskReopened {task}"),
        }
    }
}
