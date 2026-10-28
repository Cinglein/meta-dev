use std::cell::Cell;
use std::path::Path;

use meta_door::{Door, Pair};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use meta_schema::{Event, NewTask, Record};

mod malformed;

use super::GitLog;
use super::pacing::Pacing;
use crate::LogStore;
use crate::log_failure::LogFailure;

const ORIGIN: &str = "origin";
const WRITER: &str = "writer";
const DIVISION: &str = "features-and-maintenance";
const OFFLINE: [&str; 4] = ["remote", "set-url", ORIGIN, "/nonexistent/origin.git"];
const OFFLINE_PREFIX: &str = "offline: cannot fetch meta/log from origin: \"fatal: ";
const SHOWN: &str = "meta/log on origin";
const REJECTED_PREFIX: &str = "meta/log push rejected";
const REFUSE_PUSHES: [&str; 2] = ["/usr/bin/false", "pre-receive"];
const HOOKS_PATH: [&str; 2] = ["config", "core.hooksPath"];
const CONFIG: &str = "config";
const MAINTENANCE: &str = "[maintenance]\n\tauto = false\n";
const STATUS: [&str; 3] = ["status", "--porcelain", "--branch"];
const COUNT: [&str; 3] = ["rev-list", "--count", "meta/log"];
const REWIND: [&str; 3] = ["update-ref", "refs/heads/meta/log", "refs/heads/meta/log~1"];
const FIRST: usize = 0;
const SECOND: usize = 1;
const PAIR: usize = 2;
const SINGLE: usize = 1;

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    const TEMPLATE: &str = concat!("--template=", env!("CARGO_MANIFEST_DIR"), "/src/git_log/tests/template");

    fn laid(dir: &Path) {
        assert!(Self::run(dir, &["init", "-q", Self::TEMPLATE, "--bare", "."]).status.success());
    }

    fn cloned(dir: &Path, url: &str) {
        let template = tempfile::tempdir().unwrap();
        let remote = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n");
        Door::write(template.path().join(CONFIG), [MAINTENANCE, &remote].concat()).unwrap();
        let templated = format!("--template={}", template.path().display());
        assert!(Self::run(dir, &["init", "-q", &templated, "."]).status.success());
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
        Git::laid(origin.path());
        let url = origin.path().to_str().unwrap().to_owned();
        let clones =
            (0..clones).map(|_| tempfile::tempdir().unwrap()).inspect(|dir| Git::cloned(dir.path(), &url)).collect();
        Self { origin, clones }
    }

    fn log(&self, clone: usize) -> GitLog {
        GitLog {
            pacing: Pacing::Immediate,
            ..GitLog::new(self.clones[clone].path().to_owned(), ORIGIN.parse().unwrap())
        }
    }

    fn record() -> Record {
        let opened = NewTask::new(DIVISION.parse().unwrap(), WRITER.parse().unwrap(), vec![]);
        Record::new(WRITER.parse().unwrap(), Event::TaskOpened(opened))
    }

    fn observe(&self, clone: usize) -> Observation {
        let validated = Cell::new(0);
        let record = self.log(clone).append::<LogFailure>(&mut |snapshot| {
            validated.set(snapshot.records().len());
            Ok(Self::record())
        });
        Observation { validated: validated.get(), record: record.unwrap() }
    }
}

#[test]
fn writers_on_any_clone_race_and_each_lands_right_after_what_it_validated_without_touching_checkouts() {
    let fixture = Arc::new(Fixture::new(PAIR));
    let before = Git::text(fixture.clones[FIRST].path(), &STATUS);
    let seen: Arc<Mutex<Vec<Observation>>> = Arc::default();
    let writers: Vec<_> = (0..fixture.clones.len())
        .map(|clone| (Arc::clone(&fixture), Arc::clone(&seen), clone))
        .map(|(fixture, seen, clone)| std::thread::spawn(move || seen.lock().unwrap().push(fixture.observe(clone))))
        .collect();
    writers.into_iter().try_for_each(std::thread::JoinHandle::join).unwrap();
    let records = fixture.log(SECOND).read().unwrap().records().to_vec();
    assert!(seen.lock().unwrap().iter().all(|seen| records.get(seen.validated) == Some(&seen.record)));
    assert_eq!(Git::text(fixture.origin.path(), &COUNT), records.len().to_string());
    assert_eq!(
        (Git::text(fixture.clones[FIRST].path(), &STATUS), fixture.log(FIRST).to_string()),
        (before, SHOWN.to_owned())
    );
}

#[test]
fn refusals_write_nothing_and_rejected_or_offline_writes_fail() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    let refused = log.append(&mut |_| Err(LogFailure::Contended)).err().map(|failure| failure.to_string());
    assert_eq!(
        refused.zip(log.read().ok().filter(|snapshot| snapshot.records().is_empty())).map(|(refused, _)| refused),
        Some(LogFailure::Contended.to_string())
    );
    let failure = || {
        log.append::<LogFailure>(&mut |_| Ok(Fixture::record()))
            .err()
            .map(|failure| failure.to_string())
            .unwrap_or_default()
    };
    let [false_program, hook] = REFUSE_PUSHES;
    let hooks = tempfile::tempdir().unwrap();
    Door::paired(false_program, hooks.path().join(hook), Pair::Symlink).unwrap();
    let [config, key] = HOOKS_PATH;
    Git::run(fixture.origin.path(), &[config, key, &hooks.path().display().to_string()]);
    assert!(failure().starts_with(REJECTED_PREFIX));
    Git::run(fixture.clones[FIRST].path(), &OFFLINE);
    assert!(failure().starts_with(OFFLINE_PREFIX));
}

#[test]
fn a_write_after_the_remote_rewound_retries_on_the_fresh_tip_and_brings_nothing_back() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    let appended = || log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let kept = appended();
    appended();
    Git::run(fixture.origin.path(), &REWIND);
    let written = appended();
    assert_eq!(fixture.log(FIRST).read().unwrap().records(), [kept, written]);
}
