use std::fmt;
use std::str::FromStr;

use thiserror::Error;

use crate::state::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Tasks,
    Issues,
    Concerns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Open,
    All,
}

#[derive(Debug, Error)]
pub enum KindFailure {
    #[error("unknown kind `{0}`: tasks, issues or concerns")]
    Unknown(String),
}

#[derive(Debug)]
pub struct Listing<'state> {
    state: &'state State,
    kind: Kind,
    scope: Scope,
}

impl<'state> Listing<'state> {
    pub fn new(state: &'state State, kind: Kind, scope: Scope) -> Self {
        Self { state, kind, scope }
    }

    fn shown<Item>(&self, addressed: Option<Item>) -> Option<()> {
        match (self.scope, addressed) {
            (Scope::All, _) | (Scope::Open, None) => Some(()),
            (Scope::Open, Some(_)) => None,
        }
    }
}

impl FromStr for Kind {
    type Err = KindFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "tasks" => Ok(Self::Tasks),
            "issues" => Ok(Self::Issues),
            "concerns" => Ok(Self::Concerns),
            other => Err(KindFailure::Unknown(other.to_owned())),
        }
    }
}

impl fmt::Display for Listing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state;
        match self.kind {
            Kind::Tasks => state
                .tasks()
                .iter()
                .filter(|task| self.shown(task.status().done()).is_some())
                .try_for_each(|task| writeln!(formatter, "{task}")),
            Kind::Issues => state
                .issues()
                .iter()
                .filter(|issue| self.shown(state.addressed_by(issue.target())).is_some())
                .try_for_each(|issue| writeln!(formatter, "{issue}")),
            Kind::Concerns => state
                .concerns()
                .iter()
                .filter(|concern| self.shown(state.addressed_by(concern.target())).is_some())
                .try_for_each(|concern| writeln!(formatter, "{concern}")),
        }
    }
}
