use meta_log::LogStore;
use meta_schema::{Actor, Event, LevelName, Rerating, Scenario, Target};
use meta_severity::{RatingFailure, Scales};
use meta_state::{Refusal, State};

use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::output::Output;
use crate::request::writing::Writing;

#[derive(Debug)]
pub struct Rate {
    item: Target,
    level: LevelName,
    scenario: Option<Scenario>,
}

impl Rate {
    pub fn new(item: Target, level: LevelName, scenario: Option<Scenario>) -> Rate {
        Rate { item, level, scenario }
    }

    pub fn event(self, state: &State, scales: &Scales) -> Result<Event, CliFailure> {
        let rated = state.rated_item(self.item).ok_or(Refusal::UnknownTarget(self.item))?;
        let placed =
            rated.placement().division().ok_or(RatingFailure::Unscaled).map_err(CliFailure::raised::<Invalid>)?;
        let scale = scales.for_item(rated.placement()).map_err(CliFailure::raised::<Invalid>)?;
        let change = scale.change(rated.rating(), self.level, self.scenario).map_err(CliFailure::raised::<Invalid>)?;
        Ok(Event::ItemRated(Rerating::new(self.item, placed.clone(), change)))
    }

    pub fn answer(self, log: &impl LogStore, scales: &Scales, actor: Actor) -> Result<Output, CliFailure> {
        let state = State::fold(log.read()?.records());
        Writing::new(log, actor, self.event(&state, scales)?).entered()
    }
}
