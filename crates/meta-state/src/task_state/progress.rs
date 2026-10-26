mod activity;

use meta_schema::Blocker;

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::task_state::task_status::TaskStatus;
pub use activity::Activity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress {
    Active(Activity),
    Done(Stamp),
}

impl Progress {
    pub fn open() -> Self {
        Self::Active(Activity::unassigned())
    }

    pub fn assignments(&self) -> &[Stamp] {
        match self {
            Self::Active(activity) => activity.assignments(),
            Self::Done(_) => &[],
        }
    }

    pub fn done(&self) -> Option<&Stamp> {
        match self {
            Self::Done(stamp) => Some(stamp),
            Self::Active(_) => None,
        }
    }

    pub fn status(&self) -> TaskStatus {
        match self {
            Self::Done(stamp) => TaskStatus::Done(stamp.clone()),
            Self::Active(activity) => activity.status(),
        }
    }

    pub fn claimed(&self, by: Stamp, previous: Option<&meta_schema::Actor>) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.claimed(by, previous).map(Self::Active),
            Self::Done(_) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn released(&self, by: &Stamp) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.released(by.actor()).map(Self::Active),
            Self::Done(_) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn finished(&self, by: Stamp) -> Result<Self, Refusal> {
        self.released(&by).map(|_| Self::Done(by))
    }

    pub fn reopened(&self) -> Result<Activity, Refusal> {
        match self {
            Self::Done(_) => Ok(Activity::unassigned()),
            Self::Active(_) => Err(Refusal::NotDone),
        }
    }

    pub fn blocked(&self, on: Blocker) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => Ok(Self::Active(activity.blocked(on))),
            Self::Done(_) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn unblocked(&self) -> Result<Self, Refusal> {
        match self {
            Self::Active(activity) => activity.unblocked().map(Self::Active),
            Self::Done(_) => Err(Refusal::AlreadyDone),
        }
    }
}
