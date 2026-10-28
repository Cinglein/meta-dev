use std::io::Write as _;
use std::process::{Command, Stdio};

use super::scene::Scene;
use super::{FIRST, Fixture};
use crate::LogStore;
use crate::log_failure::LogFailure;

const EVENTS: &str = "events.jsonl";
const EXTRA: &str = "extra";
const REFUSED: &str = "meta/log on origin must hold only events.jsonl; nothing written";

impl Fixture {
    fn publish(&self, files: &[&str]) -> Option<LogFailure> {
        let mut import = Command::new("git")
            .arg("-C")
            .arg(self.origin.path())
            .args(["fast-import", "--quiet", "--force", "--date-format=now"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let input = import.stdin.as_mut().unwrap();
        write!(input, "commit refs/heads/meta/log\ncommitter t <t@meta.invalid> now\ndata {}\n{EXTRA}\n", EXTRA.len())
            .unwrap();
        files.iter().try_for_each(|file| write!(input, "M 100644 inline {file}\ndata 0\n")).unwrap();
        import.wait().unwrap();
        self.log(FIRST).read().err()
    }
}

pub struct MalformedLog;

impl Scene for MalformedLog {
    fn hold() {
        let fixture = Fixture::new(1);
        assert!(fixture.publish(&[EVENTS]).is_none());
        assert_eq!(fixture.publish(&[EVENTS, EXTRA]).map(|failure| failure.to_string()).as_deref(), Some(REFUSED));
        assert_eq!(fixture.publish(&[EXTRA]).map(|failure| failure.to_string()).as_deref(), Some(REFUSED));
    }
}
