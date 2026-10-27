mod unmarked;

use std::cell::RefCell;

use meta_log::LogStore;
use meta_schema::{Actor, Event, Record, SchemaVersion};
use meta_state::State;

use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::output::{Holders, Output};
use meta_severity::Scales;
use unmarked::Unmarked;

#[derive(Debug)]
pub struct Writing<'log, Log: LogStore> {
    log: &'log Log,
    actor: Actor,
    event: Event,
    guard: Option<&'log Scales>,
}

impl<'log, Log: LogStore> Writing<'log, Log> {
    pub fn new(log: &'log Log, actor: Actor, event: Event) -> Self {
        Self { log, actor, event, guard: None }
    }

    #[must_use]
    pub fn unless_stranded(self, scales: &'log Scales) -> Self {
        Self { guard: Some(scales), ..self }
    }

    fn stranding(&self, state: &State) -> Result<(), Unmarked> {
        let strands = self.guard.map(|scales| scales.stranded(state)).unwrap_or_default();
        strands.items().first().map_or(Ok(()), |_| Err(Unmarked::Failed(CliFailure::raised::<Invalid>(&strands))))
    }

    pub fn entered(self) -> Result<Output, CliFailure> {
        self.attempt().or_else(|unmarked| unmarked.recovered(|| self.after_marking()))
    }

    fn after_marking(&self) -> Result<Output, CliFailure> {
        self.marked().and_then(|()| self.attempt().map_err(CliFailure::from))
    }

    fn attempt(&self) -> Result<Output, Unmarked> {
        let holders = RefCell::new(Holders::default());
        let record = self.log.append::<Unmarked>(&mut |snapshot| {
            let record = Record::new(self.actor.clone(), self.event.clone());
            let state = State::fold(snapshot.complete()?);
            self.stranding(&state)?;
            let after = state.tried(&record)?;
            state.schema().filter(|marked| *marked >= SchemaVersion::current()).ok_or(Unmarked::Missing)?;
            holders.replace(Holders::in_state(&after, &self.event));
            Ok(record)
        })?;
        Ok(Output::Written(record, holders.take()))
    }

    fn marked(&self) -> Result<(), CliFailure> {
        self.log
            .append::<CliFailure>(&mut |snapshot| {
                snapshot.complete()?;
                Ok(Record::new(self.actor.clone(), Event::SchemaVersion(SchemaVersion::current())))
            })
            .map(|_| ())
    }
}
