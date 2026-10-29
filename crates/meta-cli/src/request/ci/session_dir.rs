mod expected_run;
mod watch_state;

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::Duration;

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_id::SessionId;
use crate::request::git_link::GitLink;
use crate::request::output::Listed;
use crate::request::project::Project;
use expected_run::ExpectedRun;
use watch_state::WatchState;

#[derive(Clone, Debug)]
pub struct SessionDir {
    own: PathBuf,
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
        let own = GitLink::common_dir(project.root())?.join(Self::ROOT).join(session.file_name());
        Some(Self { own, shared: std::env::temp_dir().join(Self::ROOT) })
    }

    pub fn record(&self, line: impl fmt::Display) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.own)?;
        std::fs::OpenOptions::new().create(true).append(true).open(self.own.join(Self::EVENTS)).and_then(|events| {
            let mut whole = std::io::BufWriter::new(events);
            writeln!(whole, "{line}")?;
            whole.flush()
        })
    }

    pub fn news(&self) -> Option<Listed> {
        let events = std::fs::read_to_string(self.own.join(Self::EVENTS)).ok()?;
        let seen: usize = std::fs::read_to_string(self.own.join(Self::SEEN))
            .ok()
            .and_then(|text| text.parse().ok())
            .unwrap_or_default();
        let lines: Vec<&str> = events.lines().collect();
        write!(std::fs::File::create(self.own.join(Self::SEEN)).ok()?, "{}", lines.len()).ok()?;
        let unseen = lines.get(seen..).filter(|unseen| !unseen.is_empty())?;
        Some(Listed::new(unseen.join("\n")))
    }

    pub fn watch(&self, commit: &Commit) -> std::io::Result<()> {
        self.settle(commit, CiStatus::Running)
    }

    pub fn settle(&self, commit: &Commit, status: CiStatus) -> std::io::Result<()> {
        let folder = self.shared.join(Self::STATES);
        std::fs::create_dir_all(&folder)?;
        let target = folder.join(commit.argument());
        let temp = target.with_extension(Self::PARTIAL);
        write!(std::fs::File::create(&temp)?, "{}", WatchState::new(commit.clone(), status))?;
        std::fs::rename(&temp, target)
    }

    pub fn status_of(&self, commit: &Commit) -> Option<CiStatus> {
        let text = std::fs::read_to_string(self.shared.join(Self::STATES).join(commit.argument())).ok()?;
        let state: WatchState = text.parse().ok()?;
        state.matching(commit).map(WatchState::status)
    }

    pub fn writable(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.own)?;
        std::fs::File::create(self.own.join(Self::PROBE))?;
        std::fs::remove_file(self.own.join(Self::PROBE))
    }

    pub fn claim(&self, commit: &Commit) -> Option<std::fs::File> {
        let folder = self.shared.join(Self::LOCKS);
        std::fs::create_dir_all(&folder).ok()?;
        let lock = std::fs::File::options().create(true).append(true).open(folder.join(commit.argument())).ok()?;
        lock.try_lock().ok().map(|()| lock)
    }

    pub fn expected(&self) -> Option<Duration> {
        let text = std::fs::read_to_string(self.own.with_file_name(Self::EXPECTED)).ok()?;
        text.parse().ok().map(ExpectedRun::duration)
    }

    pub fn learn(&self, expected: Duration) -> std::io::Result<()> {
        write!(std::fs::File::create(self.own.with_file_name(Self::EXPECTED))?, "{}", ExpectedRun::from(expected))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::SessionDir;
    use crate::request::ci::ci_status::CiStatus;
    use crate::request::ci::commit::Commit;

    const OWN: &str = "session";
    const SHA: &str = "f2e83b47f04f1aadaaf72b9d15074e933a513dc6";
    const TOOK: Duration = Duration::from_millis(1500);

    #[test]
    fn a_watched_commit_reads_as_running_and_a_learned_run_length_is_expected_next_time() {
        let root = tempfile::tempdir().unwrap();
        let dir = SessionDir { own: root.path().join(OWN), shared: root.path().to_path_buf() };
        let Ok(commit) = SHA.parse::<Commit>();
        let before = (dir.status_of(&commit), dir.expected());
        dir.watch(&commit).and_then(|()| dir.learn(TOOK)).unwrap();
        assert_eq!(before, (None, None));
        assert_eq!((dir.status_of(&commit), dir.expected()), (Some(CiStatus::Running), Some(TOOK)));
    }
}
