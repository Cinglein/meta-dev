use std::fmt;
use std::path::Path;

use crate::fixture::{Fixture, Moment};

pub const TASK: &str = "task-00000000000000a1";

pub struct SeededLog(Vec<&'static str>);

impl SeededLog {
    const EVENTS: [&'static str; 3] = [
        r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"SchemaVersion","version":2}"#,
        r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a1","text":"x","division":"features-and-maintenance","purpose":{"caused_by":{"cause":{"pull":1}}}}"#,
        r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskClaimed","task":"task-00000000000000a1"}"#,
    ];
    const BRANCH: &'static str = "refs/heads/meta/log";
    const FILE: &'static str = "events.jsonl";
    const SEEDED: Moment = Moment::after_epoch(1_700_000_000);

    pub fn through(kind: &str) -> Self {
        let tag = format!("\"kind\":\"{kind}\"");
        let last = Self::EVENTS.iter().position(|event| event.contains(&tag)).map_or(Self::EVENTS.len(), |at| at + 1);
        Self(Self::EVENTS.iter().take(last).copied().collect())
    }

    pub fn plant(&self, bare: &Path) {
        Fixture::opened(bare).written(Self::BRANCH, Self::FILE, &self.to_string(), &Self::SEEDED);
    }
}

impl fmt::Display for SeededLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|event| writeln!(formatter, "{event}"))
    }
}
