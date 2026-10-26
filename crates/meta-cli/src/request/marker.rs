use meta_log::LogStore;
use meta_schema::{Actor, Event, Record, SchemaVersion};

use crate::request::cli_failure::CliFailure;

#[derive(Debug)]
pub struct Marker;

impl Marker {
    pub fn ensured(log: &impl LogStore, actor: &Actor) -> Result<(), CliFailure> {
        log.read()?.records().iter().any(|record| record.event().schema_version().is_some()).then_some(()).map_or_else(
            || {
                log.append::<CliFailure>(&mut |snapshot| {
                    snapshot.complete()?;
                    Ok(Record::new(actor.clone(), Event::SchemaVersion(SchemaVersion::current())))
                })
                .map(|_| ())
            },
            Ok,
        )
    }
}
