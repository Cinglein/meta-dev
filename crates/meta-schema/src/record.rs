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
    use crate::{
        Block, Blocker, Claim, Event, ExternalKind, Id, NewConcern, NewIssue, NewTask, Placement, Tag, TaskLink,
        TaskRef,
    };

    const ACTOR: &str = "a";
    const DIVISION: &str = "architecture-and-planning/steward";
    const OUTSIDE: &str = "architecture";
    const TOKENS: &str = "50000";
    const DUE: &str = "2026-11-01T00:00:00Z";
    const FOREIGN_TASK_AS_CONCERN: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"ConcernIdentified","concern":"task-0000000000000000","text":"x"}"#;
    const LEGACY_TASK: &str =
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000001","text":"x"}"#;
    const LEGACY_CONCERN: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"ConcernIdentified","concern":"concern-0000000000000001","text":"x"}"#;
    const LEGACY_ISSUE: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"IssueReported","issue":"issue-0000000000000001","text":"x"}"#;
    const OUTSIDE_TASK: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000001","text":"x","division":"architecture"}"#;

    struct Samples;

    impl Samples {
        fn records() -> [Record; 8] {
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
                Event::IssueReported(NewIssue::new(division(), text())),
                Event::TaskOpened(planned),
                Event::TaskClaimed(Claim::new(task, ACTOR.parse().ok())),
                Event::TaskBlocked(Block::new(task, Blocker::External(ExternalKind::Ci))),
                Event::TaskLinked(TaskLink::new(task, "7".parse().unwrap())),
                Event::TaskDone(TaskRef::new(task)),
                Event::SchemaVersion(crate::SchemaVersion::current()),
            ]
            .map(|event| Record::new(ACTOR.parse().unwrap(), event))
        }

        fn round_tripped(records: &[Record; 8]) -> Vec<Record> {
            records
                .iter()
                .map(|record| serde_json::from_str(&serde_json::to_string(record).unwrap()).unwrap())
                .collect()
        }

        fn placements(lines: [&str; 3]) -> [Placement; 3] {
            let [task, concern, issue] = lines;
            [
                serde_json::from_str::<NewTask>(task).unwrap().placement().clone(),
                serde_json::from_str::<NewConcern>(concern).unwrap().placement().clone(),
                serde_json::from_str::<NewIssue>(issue).unwrap().placement().clone(),
            ]
        }
    }

    #[test]
    fn records_round_trip_and_reject_ids_of_the_wrong_kind() {
        let records = Samples::records();
        assert_eq!(Samples::round_tripped(&records), records);
        assert!(serde_json::from_str::<Record>(FOREIGN_TASK_AS_CONCERN).is_err());
    }

    #[test]
    fn entries_written_before_divisions_read_as_the_root_and_new_ones_keep_theirs() {
        let root = Samples::placements([LEGACY_TASK, LEGACY_CONCERN, LEGACY_ISSUE]);
        assert!(root.iter().all(|placement| *placement == Placement::default()));
        let written = serde_json::to_string(&Samples::records()[2]).unwrap();
        assert!(written.contains(DIVISION) && written.contains(TOKENS) && written.contains(DUE));
    }

    #[test]
    fn a_division_outside_the_three_top_divisions_is_refused() {
        assert!(serde_json::from_str::<Record>(OUTSIDE_TASK).is_err());
        assert!(OUTSIDE.parse::<crate::Division>().is_err());
    }
}
