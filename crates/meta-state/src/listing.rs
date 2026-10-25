use std::fmt;
use std::str::FromStr;

use meta_schema::Entry;
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

    fn rows(&self) -> Vec<String> {
        let state = self.state;
        match self.kind {
            Kind::Tasks => state.tasks().iter().filter_map(|task| self.row(task, task.entry())).collect(),
            Kind::Issues => state.issues().iter().filter_map(|issue| self.row(issue, issue.target().entry())).collect(),
            Kind::Concerns => {
                state.concerns().iter().filter_map(|concern| self.row(concern, concern.target().entry())).collect()
            }
        }
    }

    fn row(&self, item: &dyn fmt::Display, entry: Entry) -> Option<String> {
        let marker = self.state.retirement(entry).map(ToString::to_string).unwrap_or_default();
        self.shown(self.state.closed(entry)).map(|()| format!("{item}{marker}"))
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
        self.rows().iter().try_for_each(|row| writeln!(formatter, "{row}"))
    }
}
