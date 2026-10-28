mod expected_run;
mod watch_state;

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use meta_door::kinds::Handle;

use meta_door::{Change, Door, Entry, MetaStore, Opening, Pair};

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_id::SessionId;
use crate::request::output::Listed;
use crate::request::project::Project;
use expected_run::ExpectedRun;
use watch_state::WatchState;

#[derive(Clone, Debug)]
pub struct SessionDir {
    own: Entry,
    expected: Entry,
    shared: PathBuf,
}

impl SessionDir {
    const ROOT: &'static str = "meta-ci";
    const EVENTS: &'static str = "events";
    const SEEN: &'static str = "seen";
    const STATES: &'static str = "states";
    const PARTIAL: &'static str = "partial";
    const LOCKS: &'static str = "locks";
    const PROBE: &'static str = "probe";
    const EXPECTED: &'static str = "expected";

    pub fn for_session(project: &Project, session: &SessionId) -> Option<Self> {
        let store = MetaStore::located(project.root()).ok()?;
        let own = store.entry(Path::new(Self::ROOT).join(session.file_name()));
        let expected = store.entry(Path::new(Self::ROOT).join(Self::EXPECTED));
        Some(Self { own, expected, shared: std::env::temp_dir().join(Self::ROOT) })
    }

    pub fn record(&self, line: impl fmt::Display) -> std::io::Result<()> {
        Door::changed(self.own.clone(), Change::DirAll)?;
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
        let folder = self.shared.join(Self::STATES);
        Door::changed(&folder, Change::DirAll)?;
        let target = folder.join(commit.argument());
        let temp = target.with_extension(Self::PARTIAL);
        write!(Door::opened(&temp, Opening::Truncate)?, "{}", WatchState::new(commit.clone(), status))?;
        Door::paired(temp, target, Pair::Rename)
    }

    pub fn status_of(&self, commit: &Commit) -> Option<CiStatus> {
        let text = Door::read(self.shared.join(Self::STATES).join(commit.argument())).ok()?;
        let state: WatchState = text.parse().ok()?;
        state.matching(commit).map(WatchState::status)
    }

    pub fn writable(&self) -> std::io::Result<()> {
        Door::changed(self.own.clone(), Change::DirAll)?;
        Door::opened(self.named(Self::PROBE), Opening::Truncate)?;
        Door::changed(self.named(Self::PROBE), Change::RemoveFile)
    }

    pub fn claim(&self, commit: &Commit) -> Option<Handle> {
        let folder = self.shared.join(Self::LOCKS);
        Door::changed(&folder, Change::DirAll).ok()?;
        let lock = Door::opened(folder.join(commit.argument()), Opening::Append).ok()?;
        lock.try_lock().ok().map(|()| lock)
    }

    pub fn expected(&self) -> Option<Duration> {
        let text = Door::read(self.expected.clone()).ok()?;
        text.parse().ok().map(ExpectedRun::duration)
    }

    pub fn learn(&self, expected: Duration) -> std::io::Result<()> {
        write!(Door::opened(self.expected.clone(), Opening::Truncate)?, "{}", ExpectedRun::from(expected))
    }

    fn named(&self, name: &str) -> Entry {
        self.own.map(|own| own.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::{CiStatus, Commit, Entry, SessionDir};

    #[test]
    fn a_watched_commit_reads_back_as_running() {
        let dir = tempfile::tempdir().unwrap();
        let own = Entry::from(dir.path().join("own"));
        let session = SessionDir { own: own.clone(), expected: own, shared: dir.path().join("shared") };
        let Ok(commit): Result<Commit, _> = "0123456789abcdef0123456789abcdef01234567".parse();
        session.watch(&commit).unwrap();
        assert_eq!(session.status_of(&commit), Some(CiStatus::Running));
    }
}
