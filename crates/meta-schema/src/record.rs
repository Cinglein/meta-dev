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
    use meta_laws::{Laws, laws};
    use proptest::sample::select;

    use super::Record;
    use crate::{
        Block, Blocker, Claim, Event, ExternalKind, Id, NewConcern, NewIssue, NewTask, Tag, TaskLink, TaskRef,
    };

    const ACTOR: &str = "a";
    const DIVISION: &str = "architecture-and-planning/steward";
    const TOKENS: &str = "50000";
    const DUE: &str = "2026-11-01T00:00:00Z";
    const REFUSED: [&str; 2] = [
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"ConcernIdentified","concern":"task-0000000000000000","text":"x"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000001","text":"x","division":"architecture"}"#,
    ];

    struct Samples;

    impl Samples {
        fn records() -> Vec<Record> {
            let task = Id::generate();
            let text = || ACTOR.parse().unwrap();
            let division = || DIVISION.parse().unwrap();
            let planned =
                NewTask::new(division(), text(), vec![]).allowing(TOKENS.parse().unwrap()).due_by(DUE.parse().unwrap());
            [
                Event::ConcernIdentified(NewConcern::new(
                    division(),
                    text(),
                    Some(text()),
                    vec![Tag::AcceptanceCandidate],
                )),
                Event::IssueReported(NewIssue::new(division(), text()).rated(ACTOR.parse().unwrap())),
                Event::TaskOpened(planned),
                Event::TaskClaimed(Claim::new(task, ACTOR.parse().ok())),
                Event::TaskBlocked(Block::new(task, Blocker::External(ExternalKind::Ci))),
                Event::TaskLinked(TaskLink::new(task, "7".parse().unwrap())),
                Event::TaskDone(TaskRef::new(task)),
                Event::SchemaVersion(crate::SchemaVersion::current()),
            ]
            .map(|event| Record::new(ACTOR.parse().unwrap(), event))
            .into()
        }
    }

    laws! {
        records_are_stored: Laws::recorded::<Record> => select(Samples::records());
    }

    #[test]
    fn records_refuse_ids_of_the_wrong_kind_and_divisions_outside_the_three_top_ones() {
        assert!(REFUSED.iter().all(|line| serde_json::from_str::<Record>(line).is_err()));
    }
}
