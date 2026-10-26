use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::event::Event;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    at: DateTime<Utc>,
    actor: Actor,
    #[serde(flatten)]
    event: Event,
}

impl Record {
    #[must_use]
    pub fn new(actor: Actor, event: Event) -> Self {
        Self { at: Utc::now(), actor, event }
    }

    #[must_use]
    pub fn written_at(&self) -> DateTime<Utc> {
        self.at
    }

    #[must_use]
    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    #[must_use]
    pub fn event(&self) -> &Event {
        &self.event
    }
}

impl fmt::Display for Record {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.at.format("%Y-%m-%d %H:%MZ"), self.actor, self.event)
    }
}

#[cfg(test)]
mod tests {
    use super::Record;
    use crate::{Claim, Event, Id, NewConcern, NewTask, Tag, TaskRef};

    const ACTOR: &str = "a";
    const FOREIGN_TASK_AS_CONCERN: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"ConcernIdentified","concern":"task-0000000000000000","text":"x"}"#;

    struct Samples;

    impl Samples {
        fn records() -> [Record; 4] {
            let task = Id::generate();
            let text = || ACTOR.parse().unwrap();
            [
                Event::ConcernIdentified(NewConcern::new(text(), Some(text()), vec![Tag::AcceptanceCandidate])),
                Event::TaskOpened(NewTask::new(text(), vec![])),
                Event::TaskClaimed(Claim::new(task, ACTOR.parse().ok())),
                Event::TaskDone(TaskRef::new(task)),
            ]
            .map(|event| Record::new(ACTOR.parse().unwrap(), event))
        }

        fn round_tripped(records: &[Record; 4]) -> Vec<Record> {
            records
                .iter()
                .map(|record| serde_json::from_str(&serde_json::to_string(record).unwrap()).unwrap())
                .collect()
        }
    }

    #[test]
    fn records_round_trip_and_reject_ids_of_the_wrong_kind() {
        let records = Samples::records();
        assert_eq!(Samples::round_tripped(&records), records);
        assert!(serde_json::from_str::<Record>(FOREIGN_TASK_AS_CONCERN).is_err());
    }
}
