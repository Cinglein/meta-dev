use std::fmt;

use meta_schema::Entry;
use meta_state::State;

use crate::kind::Kind;
use crate::scope::Scope;

#[derive(Debug)]
pub struct Listing<'state> {
    state: &'state State,
    kind: Kind,
    scope: Scope,
}

impl<'state> Listing<'state> {
    #[must_use]
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
            Kind::Projects => state.projects().iter().map(ToString::to_string).collect(),
        }
    }

    fn row(&self, item: &dyn fmt::Display, entry: Entry) -> Option<String> {
        let marker = self.state.retirement(entry).map(ToString::to_string).unwrap_or_default();
        (self.scope == Scope::All || self.state.closed(entry).is_none()).then(|| format!("{item}{marker}"))
    }
}

impl fmt::Display for Listing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rows().iter().try_for_each(|row| writeln!(formatter, "{row}"))
    }
}

#[cfg(test)]
mod tests;
