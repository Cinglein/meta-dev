mod activity;

use meta_schema::{Actor, Block, Blocker, CommitId};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::task_state::reopening::Reopening;
use crate::task_state::task_status::TaskStatus;
pub use activity::Activity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress {
    Active(Activity),
    Done(Stamp, Vec<Stamp>),
}

impl Progress {
    pub fn open() -> Self {
        Self::Active(Activity::unassigned())
    }

    pub fn assignments(&self) -> &[Stamp] {
        match self {
            Self::Active(activity) => activity.assignments(),
            Self::Done(..) => &[],
        }
    }

    pub fn done(&self) -> Option<&Stamp> {
        match self {
            Self::Done(stamp, _) => Some(stamp),
            Self::Active(_) => None,
        }
    }

    pub fn status(&self) -> TaskStatus {
        match self {
            Self::Done(stamp, _) => TaskStatus::Done(stamp.clone()),
            Self::Active(activity) => activity.status(),
        }
    }

    pub fn ended(&self) -> Option<Blocker> {
        match self {
            Self::Active(activity) => activity.ended(),
            Self::Done(..) => None,
        }
    }

    pub fn dissolved(&self) -> Self {
        match self {
            Self::Active(activity) => Self::Active(activity.dissolved()),
            Self::Done(..) => self.clone(),
        }
    }

    pub fn claimed(&self, by: Stamp, previous: Option<&Actor>) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.claimed(by, previous).map(Self::Active),
            Self::Done(..) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn released(&self, by: &Stamp) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.released(by).map(Self::Active),
            Self::Done(..) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn finished(&self, by: Stamp) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => {
                activity.holder(by.actor()).map(|()| Self::Done(by, activity.assignments().to_vec()))
            }
            Self::Done(..) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn reopened(&self, reopening: Reopening) -> Result<Activity, Refusal> {
        match self {
            Self::Done(_, held) => Ok(reopening.activity(held)),
            Self::Active(_) => Err(Refusal::NotDone),
        }
    }

    pub fn blocked(&self, block: &Block, by: &Actor) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.blocked(block, by).map(Self::Active),
            Self::Done(..) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn waited_at(&self) -> Option<&CommitId> {
        match self {
            Self::Active(activity) => activity.waited_at(),
            Self::Done(..) => None,
        }
    }

    pub fn unblocked(&self, by: &Actor) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.unblocked(by).map(Self::Active),
            Self::Done(..) => Err(Refusal::AlreadyDone),
        }
    }
}
