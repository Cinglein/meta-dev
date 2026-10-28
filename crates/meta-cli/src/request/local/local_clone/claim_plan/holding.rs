use meta_schema::{Id, Record, Task};
use meta_state::State;

use crate::request::cli_failure::CliFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holding {
    Held,
    Fresh,
}

impl Holding {
    pub fn vetted(state: &State, claim: &Record, task: Id<Task>) -> Result<Self, CliFailure> {
        let held = state.holders(task).contains(claim.actor()).then_some(Self::Held);
        held.map_or_else(|| state.vet(claim).map(|()| Self::Fresh).map_err(CliFailure::from), Ok)
    }
}
