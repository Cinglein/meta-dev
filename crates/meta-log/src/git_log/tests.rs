use std::cell::Cell;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use meta_git::{FirstParents, Layout, ObjectId, Reads, RefMove, RefName, RefTarget, Repository, Writes};
use meta_schema::{Event, NewTask, Record};

mod fast_forward;
mod malformed;
mod staged;

use super::pacing::Pacing;
use super::{Budget, GitLog, RemoteHeads};
use crate::LogStore;
use crate::log_failure::LogFailure;

const ORIGIN: &str = "origin";
const WRITER: &str = "writer";
const DIVISION: &str = "features-and-maintenance";
const NOWHERE: &str = "/nonexistent/origin.git";
const OFFLINE_PREFIX: &str = "offline: cannot fetch meta/log from origin: \"";
const SHOWN: &str = "meta/log on origin";
const REJECTED_PREFIX: &str = "meta/log push rejected";
const HELD_REF: &str = "refs/heads/meta/log.lock";
const REFUSE_PUSHES: [&str; 2] = ["/usr/bin/false", "hooks/pre-receive"];
const BRANCH: &str = "refs/heads/meta/log";
const HEAD: &str = "HEAD";
const INDEX: &str = ".git/index";
const FIRST: usize = 0;
const SECOND: usize = 1;
const PAIR: usize = 2;
const SINGLE: usize = 1;
const PATIENT: Duration = Duration::from_secs(30);
const ORIGIN_CONFIG_FILE: &str = "config";
const ORIGIN_CONFIG: &str = "[core]\n\trepositoryformatversion = 0\n\tbare = true\n[maintenance]\n\tauto = false\n";

struct Port;

impl Port {
    fn linked(dir: &Path, url: &str) {
        let config = format!(
            "[core]\n\trepositoryformatversion = 0\n\tbare = false\n[maintenance]\n\tauto = false\n[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n"
        );
        std::fs::write(dir.join(".git/config"), config).unwrap();
    }

    fn target(dir: &Path, name: &str) -> Option<ObjectId> {
        let Ok(name) = name.parse::<RefName>();
        Repository::open(dir).unwrap().read(&RefTarget::from(name)).unwrap()
    }

    fn history(dir: &Path) -> Vec<ObjectId> {
        Self::target(dir, BRANCH)
            .map(|tip| Repository::open(dir).unwrap().read(&FirstParents::from(tip)).unwrap())
            .unwrap_or_default()
    }
}

struct Guards;

impl Guards {
    fn held(origin: &Path) -> std::path::PathBuf {
        let lock = origin.join(HELD_REF);
        std::fs::create_dir_all(lock.parent().unwrap()).unwrap();
        std::fs::write(&lock, "").unwrap();
        lock
    }

    fn hooked(origin: &Path) {
        let [false_program, hook] = REFUSE_PUSHES;
        std::fs::create_dir_all(origin.join(hook).parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(false_program, origin.join(hook)).unwrap();
    }
}

impl Port {
    fn untouched(dir: &Path) -> bool {
        Self::target(dir, HEAD).is_none() && !dir.join(INDEX).exists()
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
        Repository::created(origin.path(), Layout::Bare).unwrap();
        std::fs::write(origin.path().join(ORIGIN_CONFIG_FILE), ORIGIN_CONFIG).unwrap();
        let url = origin.path().to_str().unwrap().to_owned();
        let clones = (0..clones)
            .map(|_| tempfile::tempdir().unwrap())
            .inspect(|dir| {
                Repository::created(dir.path(), Layout::Working).unwrap();
                Port::linked(dir.path(), &url);
            })
            .collect();
        Self { origin, clones }
    }

    fn log(&self, clone: usize) -> GitLog {
        let budget = Budget::new(PATIENT);
        GitLog {
            pacing: Pacing::Immediate,
            ..GitLog::new(self.clones[clone].path().to_owned(), ORIGIN.parse().unwrap(), budget).unwrap()
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
    let seen: Arc<Mutex<Vec<Observation>>> = Arc::default();
    let writers: Vec<_> = (0..fixture.clones.len())
        .map(|clone| (Arc::clone(&fixture), Arc::clone(&seen), clone))
        .map(|(fixture, seen, clone)| std::thread::spawn(move || seen.lock().unwrap().push(fixture.observe(clone))))
        .collect();
    writers.into_iter().try_for_each(std::thread::JoinHandle::join).unwrap();
    let records = fixture.log(SECOND).read().unwrap().records().to_vec();
    assert!(seen.lock().unwrap().iter().all(|seen| records.get(seen.validated) == Some(&seen.record)));
    assert_eq!(Port::history(fixture.origin.path()).len(), records.len());
    assert_eq!(
        (Port::untouched(fixture.clones[FIRST].path()), fixture.log(FIRST).to_string()),
        (true, SHOWN.to_owned())
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
    let failure = || log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).err().map(|f| f.to_string());
    let held = Guards::held(fixture.origin.path());
    let locked = failure().unwrap_or_default();
    std::fs::remove_file(held).unwrap();
    Guards::hooked(fixture.origin.path());
    let hooked = failure().unwrap_or_default();
    assert!(locked.starts_with(REJECTED_PREFIX) && hooked.starts_with(REJECTED_PREFIX), "{locked} {hooked}");
    Port::linked(fixture.clones[FIRST].path(), NOWHERE);
    let offline = failure().unwrap_or_default();
    assert!(offline.starts_with(OFFLINE_PREFIX) && offline.contains(NOWHERE), "{offline}");
}

#[test]
fn a_write_after_the_remote_rewound_fast_forwards_and_restores_every_record_it_knew() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    let appended = || log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let kept = vec![appended(), appended()];
    let earlier = Port::history(fixture.origin.path())[SECOND];
    let origin = Repository::open(fixture.origin.path()).unwrap();
    let latest = Port::target(fixture.origin.path(), BRANCH);
    origin.write(&RefMove::new(BRANCH.parse().unwrap(), latest, earlier)).unwrap();
    let written = [kept.clone(), vec![appended()]].concat();
    let tip = Port::target(fixture.origin.path(), BRANCH).unwrap();
    let heads: RemoteHeads = format!("{tip}\t{BRANCH}\n").parse().unwrap();
    let [read, listed] =
        [fixture.log(FIRST).read(), log.read_listed(&heads)].map(|read| read.unwrap().records().to_vec());
    assert_eq!((read, listed), (written.clone(), written));
}
