use std::fmt;

use meta_log::LogStore;
use meta_schema::{Actor, Event};
use meta_state::State;

use crate::request::cli_failure::CliFailure;

#[derive(Debug, Default)]
pub struct Holders(Vec<Actor>);

impl Holders {
    pub fn after(log: &impl LogStore, event: &Event) -> Result<Self, CliFailure> {
        let held = event
            .claimed_task()
            .map(|task| log.read().map(|snapshot| State::fold(snapshot.records()).holders(task)))
            .transpose()?;
        Ok(Self(held.unwrap_or_default()))
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
