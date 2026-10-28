mod watch_state;

use std::fmt;
use std::io::Write as _;
use std::path::Path;

use std::fs::File;

use meta_door::{Change, Door, Entry, MetaStore, Opening, Pair};

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_id::SessionId;
use crate::request::output::Listed;
use crate::request::project::Project;
use watch_state::WatchState;

#[derive(Clone, Debug)]
pub struct SessionDir(Entry);

impl SessionDir {
    const ROOT: &'static str = "meta-ci";
    const EVENTS: &'static str = "events";
    const SEEN: &'static str = "seen";
    const STATES: &'static str = "states";
    const PARTIAL: &'static str = "partial";
    const LOCKS: &'static str = "locks";
    const PROBE: &'static str = "probe";

    pub fn for_session(project: &Project, session: &SessionId) -> Option<Self> {
        let store = MetaStore::located(project.root()).ok()?;
        Some(Self(store.entry(Path::new(Self::ROOT).join(session.file_name()))))
    }

    pub fn record(&self, line: impl fmt::Display) -> std::io::Result<()> {
        Door::changed(self.0.clone(), Change::DirAll)?;
        Door::opened(self.named(Self::EVENTS), Opening::Append).and_then(|events| {
            let mut whole = std::io::BufWriter::new(events);
            writeln!(whole, "{line}")?;
            whole.flush()
        })
    }

    pub fn news(&self) -> Option<Listed> {
        let events = Door::read(self.named(Self::EVENTS)).ok()?;
        let seen: usize =
            Door::read(self.named(Self::SEEN)).ok().and_then(|text| text.parse().ok()).unwrap_or_default();
        let lines: Vec<&str> = events.lines().collect();
        write!(Door::opened(self.named(Self::SEEN), Opening::Truncate).ok()?, "{}", lines.len()).ok()?;
        let unseen = lines.get(seen..).filter(|unseen| !unseen.is_empty())?;
        Some(Listed::new(unseen.join("\n")))
    }

    pub fn watch(&self, commit: &Commit) -> std::io::Result<()> {
        self.settle(commit, CiStatus::Running)
    }

    pub fn settle(&self, commit: &Commit, status: CiStatus) -> std::io::Result<()> {
        Door::changed(self.named(Self::STATES), Change::DirAll)?;
        let target = self.named(Self::STATES).map(|states| states.join(commit.argument()));
        let temp = target.map(|target| target.with_extension(Self::PARTIAL));
        write!(Door::opened(temp.clone(), Opening::Truncate)?, "{}", WatchState::new(commit.clone(), status))?;
        Door::paired(temp, target, Pair::Rename)
    }

    pub fn status_of(&self, commit: &Commit) -> Option<CiStatus> {
        let text = Door::read(self.named(Self::STATES).map(|states| states.join(commit.argument()))).ok()?;
        let state: WatchState = text.parse().ok()?;
        state.matching(commit).map(WatchState::status)
    }

    pub fn writable(&self) -> std::io::Result<()> {
        Door::changed(self.0.clone(), Change::DirAll)?;
        Door::opened(self.named(Self::PROBE), Opening::Truncate)?;
        Door::changed(self.named(Self::PROBE), Change::RemoveFile)
    }

    pub fn claim(&self, commit: &Commit) -> Option<File> {
        Door::changed(self.named(Self::LOCKS), Change::DirAll).ok()?;
        let lock =
            Door::opened(self.named(Self::LOCKS).map(|locks| locks.join(commit.argument())), Opening::Append).ok()?;
        lock.try_lock().ok().map(|()| lock)
    }

    fn named(&self, name: &str) -> Entry {
        self.0.map(|session| session.join(name))
    }
}
