use std::fmt;

use meta_schema::Actor;

use crate::refusal::Refusal;
use crate::stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Claimed(Stamp),
    Done(Stamp),
}

impl TaskStatus {
    pub fn claimed(&self, by: Stamp, previous: Option<&Actor>) -> Result<Self, Refusal> {
        match (self, previous) {
            (Self::Open, None) => Ok(Self::Claimed(by)),
            (Self::Open, Some(named)) => Err(Refusal::NotClaimant(named.clone())),
            (Self::Claimed(held), Some(named)) if named == held.actor() && held.actor() != by.actor() => {
                Ok(Self::Claimed(by))
            }
            (Self::Claimed(held), Some(named)) if named != held.actor() => Err(Refusal::NotClaimant(named.clone())),
            (Self::Claimed(held), _) => Err(Refusal::ClaimedBy(held.actor().clone())),
            (Self::Done(_), _) => Err(Refusal::AlreadyDone),
        }
    }

    pub fn released(&self, by: &Stamp) -> Result<Self, Refusal> {
        self.held_by(by).map(|_| Self::Open)
    }

    pub fn finished(&self, by: Stamp) -> Result<Self, Refusal> {
        self.held_by(&by).map(|_| Self::Done(by))
    }

    pub fn reopened(&self) -> Result<Self, Refusal> {
        match self {
            Self::Done(_) => Ok(Self::Open),
            Self::Open | Self::Claimed(_) => Err(Refusal::NotDone),
        }
    }

    pub fn done(&self) -> Option<&Stamp> {
        match self {
            Self::Done(stamp) => Some(stamp),
            Self::Open | Self::Claimed(_) => None,
        }
    }

    pub fn unfinished(&self) -> Option<&Self> {
        match self {
            Self::Done(_) => None,
            Self::Open | Self::Claimed(_) => Some(self),
        }
    }

    pub fn claimant(&self) -> Option<&Stamp> {
        match self {
            Self::Claimed(stamp) => Some(stamp),
            Self::Open | Self::Done(_) => None,
        }
    }

    fn held_by(&self, by: &Stamp) -> Result<&Stamp, Refusal> {
        match self {
            Self::Claimed(held) if held.actor() == by.actor() => Ok(held),
            Self::Claimed(held) => Err(Refusal::HeldBy(held.actor().clone())),
            Self::Open => Err(Refusal::Unclaimed),
            Self::Done(_) => Err(Refusal::AlreadyDone),
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => formatter.write_str("open"),
            Self::Claimed(stamp) => write!(formatter, "claimed by {stamp}"),
            Self::Done(stamp) => write!(formatter, "done by {stamp}"),
        }
    }
}
