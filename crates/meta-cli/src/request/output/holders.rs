use std::fmt;

use meta_schema::{Actor, Event};
use meta_state::State;

#[derive(Debug, Default)]
pub struct Holders(Vec<Actor>);

impl Holders {
    pub fn in_state(state: &State, event: &Event) -> Self {
        Self(event.claimed_task().map(|task| state.holders(task)).unwrap_or_default())
    }
}

impl fmt::Display for Holders {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        (self.0.len() > 1)
            .then_some(())
            .map_or(Ok(()), |()| writeln!(formatter, "notice: held by {}", names.join(", ")))
    }
}
