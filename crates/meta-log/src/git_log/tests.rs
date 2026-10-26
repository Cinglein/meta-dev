use std::cell::Cell;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use meta_schema::{Event, NewTask, Record};

use super::GitLog;
use crate::LogStore;
use crate::log_failure::LogFailure;

const ORIGIN: &str = "origin";
const WRITER: &str = "writer";
const OFFLINE: [&str; 4] = ["remote", "set-url", ORIGIN, "/nonexistent/origin.git"];
const OFFLINE_PREFIX: &str = "offline";
const SHOWN: &str = "meta/log on origin";
const REJECTED_PREFIX: &str = "meta/log push rejected";
const REFUSE_PUSHES: [&str; 2] = ["/usr/bin/false", "hooks/pre-receive"];
const STATUS: [&str; 3] = ["status", "--porcelain", "--branch"];
const COUNT: [&str; 3] = ["rev-list", "--count", "meta/log"];
const FIRST: usize = 0;
const SECOND: usize = 1;
const INTERLEAVED: [usize; 3] = [FIRST, SECOND, FIRST];
const PAIR: usize = 2;
const SINGLE: usize = 1;

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn link_origin(dir: &Path, url: &str) {
        let remote = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n");
        std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(".git/config"))
            .unwrap()
            .write_all(remote.as_bytes())
            .unwrap();
    }

    fn text(dir: &Path, words: &[&str]) -> String {
        String::from_utf8_lossy(&Self::run(dir, words).stdout).trim().to_owned()
    }
}

struct Observation {
    validated: usize,
    record: Record,
}

struct Fixture {
    origin: tempfile::TempDir,
    clones: Vec<tempfile::TempDir>,
}

impl Fixture {
    fn new(clones: usize) -> Self {
        let origin = tempfile::tempdir().unwrap();
        Git::run(origin.path(), &["init", "-q", "--bare"]);
        let url = origin.path().to_str().unwrap().to_owned();
        let clones = (0..clones)
            .map(|_| tempfile::tempdir().unwrap())
            .inspect(|dir| {
                Git::run(dir.path(), &["init", "-q"]);
                Git::link_origin(dir.path(), &url);
            })
            .collect();
        Self { origin, clones }
    }

    fn log(&self, clone: usize) -> GitLog {
        GitLog::new(self.clones[clone].path().to_owned(), &ORIGIN)
    }

    fn record() -> Record {
        Record::new(WRITER.parse().unwrap(), Event::TaskOpened(NewTask::new(WRITER.parse().unwrap(), vec![])))
    }

    fn observe(&self, clone: usize) -> Observation {
        let validated = Cell::new(0);
        let record = self.log(clone).append(&mut |snapshot| {
            validated.set(snapshot.records().len());
            Ok(Self::record())
        });
        Observation { validated: validated.get(), record: record.unwrap() }
    }
}

#[test]
fn appends_from_any_clone_land_on_the_remote_in_commit_order_without_touching_checkouts() {
    let fixture = Fixture::new(PAIR);
    let before = Git::text(fixture.clones[FIRST].path(), &STATUS);
    let written: Vec<Record> = INTERLEAVED.map(|clone| fixture.observe(clone).record).to_vec();
    assert_eq!(fixture.log(SECOND).read().unwrap().records(), written);
    assert_eq!(Git::text(fixture.clones[FIRST].path(), &STATUS), before);
    assert_eq!(Git::text(fixture.origin.path(), &COUNT), written.len().to_string());
}

#[test]
fn racing_writers_each_land_right_after_what_they_validated() {
    let fixture = Arc::new(Fixture::new(PAIR));
    let seen: Arc<Mutex<Vec<Observation>>> = Arc::default();
    let writers: Vec<_> = (0..fixture.clones.len())
        .map(|clone| (Arc::clone(&fixture), Arc::clone(&seen), clone))
        .map(|(fixture, seen, clone)| std::thread::spawn(move || seen.lock().unwrap().push(fixture.observe(clone))))
        .collect();
    writers.into_iter().try_for_each(std::thread::JoinHandle::join).unwrap();
    let records = fixture.log(FIRST).read().unwrap().records().to_vec();
    assert!(seen.lock().unwrap().iter().all(|seen| records.get(seen.validated) == Some(&seen.record)));
    assert_eq!(fixture.log(FIRST).to_string(), SHOWN);
}

#[test]
fn refusals_write_nothing_and_rejected_or_offline_writes_fail() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    let refused = log.append(&mut |_| Err(LogFailure::refused(WRITER))).err().map(|failure| failure.to_string());
    assert_eq!(
        refused.zip(log.read().ok().filter(|snapshot| snapshot.records().is_empty())).map(|(refused, _)| refused),
        Some(format!("refused: {WRITER}"))
    );
    let failure =
        || log.append(&mut |_| Ok(Fixture::record())).err().map(|failure| failure.to_string()).unwrap_or_default();
    let [false_program, hook] = REFUSE_PUSHES;
    std::os::unix::fs::symlink(false_program, fixture.origin.path().join(hook)).unwrap();
    assert!(failure().starts_with(REJECTED_PREFIX));
    Git::run(fixture.clones[FIRST].path(), &OFFLINE);
    assert!(failure().starts_with(OFFLINE_PREFIX));
}
