use meta_git::{FileCommit, Files, Identity, RefMove, Repository, Writes};

use super::{BRANCH, FIRST, Fixture};
use crate::LogStore;
use crate::log_failure::LogFailure;

const EVENTS: &str = "events.jsonl";
const EXTRA: &str = "extra";
const REFUSED: &str = "meta/log on origin must hold only events.jsonl; nothing written";

impl Fixture {
    fn publish(&self, files: &[&str]) -> Option<LogFailure> {
        let origin = Repository::open(self.origin.path()).unwrap();
        let files: Files = files.iter().map(|file| ((*file).to_owned(), Some(String::new()))).collect();
        let author = Identity::new(EXTRA.to_owned(), EXTRA.to_owned());
        let commit = origin.write(&FileCommit::new(None, files, author, EXTRA.to_owned())).unwrap();
        origin.write(&RefMove::new(BRANCH.parse().unwrap(), commit)).unwrap();
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
