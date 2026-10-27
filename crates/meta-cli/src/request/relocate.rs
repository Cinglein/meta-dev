use meta_log::LogStore;
use meta_schema::{Actor, Division, Event, LevelName, Relocation, Scenario, Target};
use meta_state::{Refusal, State};

use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::output::Output;
use crate::request::scale_policy::ScalePolicy;
use crate::request::writing::Writing;

#[derive(Debug)]
pub struct Relocate {
    item: Target,
    division: Division,
    level: Option<LevelName>,
    scenario: Option<Scenario>,
}

impl Relocate {
    pub fn new(item: Target, division: Division, level: Option<LevelName>, scenario: Option<Scenario>) -> Relocate {
        Relocate { item, division, level, scenario }
    }

    pub fn event(self, state: &State, policy: &ScalePolicy) -> Result<Event, CliFailure> {
        policy.known(&self.division).map_err(CliFailure::raised::<Invalid>)?;
        let rated = state.rated_item(self.item).ok_or(Refusal::UnknownTarget(self.item))?;
        let carried = policy.scales().carried(rated.rating(), &self.division, self.level, self.scenario);
        let change = carried.map_err(CliFailure::raised::<Invalid>)?;
        Ok(Event::ItemMoved(Relocation::new(self.item, rated.placement().clone(), self.division, change)))
    }

    pub fn answer(self, log: &impl LogStore, policy: &ScalePolicy, actor: Actor) -> Result<Output, CliFailure> {
        let state = State::fold(log.read()?.records());
        Writing::new(log, actor, self.event(&state, policy)?).entered()
    }
}
