use std::fs;

use super::{FIRST, Fixture, Git};
use crate::LogStore;
use crate::log_failure::LogFailure;

const EVENTS: &str = "events.jsonl";
const EXTRA: &str = "extra";
const REFUSED: &str = "meta/log on origin must hold only events.jsonl; nothing written";

impl Fixture {
    fn publish(&self, files: &[&str]) -> Option<LogFailure> {
        let clone = self.clones[FIRST].path();
        Git::run(clone, &["checkout", "-q", "--orphan", "meta/log"]);
        Git::run(clone, &["rm", "-q", "-r", "-f", "--ignore-unmatch", "."]);
        files.iter().try_for_each(|file| fs::write(clone.join(file), "")).unwrap();
        Git::run(clone, &["add", "-A"]);
        Git::run(clone, &["commit", "-q", "-m", EXTRA]);
        Git::run(clone, &["push", "-q", "-f", "origin", "HEAD:refs/heads/meta/log"]);
        self.log(FIRST).read().err()
    }
}

#[test]
fn a_meta_log_without_exactly_events_jsonl_is_refused() {
    let fixture = Fixture::new(1);
    assert!(fixture.publish(&[EVENTS]).is_none());
    assert_eq!(fixture.publish(&[EVENTS, EXTRA]).map(|failure| failure.to_string()).as_deref(), Some(REFUSED));
    assert_eq!(fixture.publish(&[EXTRA]).map(|failure| failure.to_string()).as_deref(), Some(REFUSED));
}
