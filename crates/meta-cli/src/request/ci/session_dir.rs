mod git_link;
mod watch_state;

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_id::SessionId;
use crate::request::output::Listed;
use crate::request::project::Project;
use git_link::GitLink;
use watch_state::WatchState;

#[derive(Clone, Debug)]
pub struct SessionDir(PathBuf);

impl SessionDir {
    const ROOT: &'static str = "meta-ci";
    const EVENTS: &'static str = "events";
    const SEEN: &'static str = "seen";
    const STATE: &'static str = "state";

    pub fn for_session(project: &Project, session: &SessionId) -> Option<Self> {
        GitLink::common_dir(project.root()).map(|base| Self(base.join(Self::ROOT).join(session.file_name())))
    }

    pub fn record(&self, line: impl fmt::Display) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.0.join(Self::EVENTS))
            .and_then(|mut events| writeln!(events, "{line}"))
    }

    pub fn news(&self) -> Option<Listed> {
        let events = std::fs::read_to_string(self.0.join(Self::EVENTS)).ok()?;
        let seen: usize = std::fs::read_to_string(self.0.join(Self::SEEN))
            .ok()
            .and_then(|text| text.parse().ok())
            .unwrap_or_default();
        let lines: Vec<&str> = events.lines().collect();
        write!(std::fs::File::create(self.0.join(Self::SEEN)).ok()?, "{}", lines.len()).ok()?;
        let unseen = lines.get(seen..).filter(|unseen| !unseen.is_empty())?;
        Some(Listed::new(unseen.join("\n")))
    }

    pub fn watch(&self, commit: &Commit) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        std::fs::remove_file(self.0.join(Self::STATE))
            .or_else(|error| if error.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(error) })?;
        self.save(&WatchState::new(commit.clone(), CiStatus::Running))
    }

    pub fn settle(&self, commit: &Commit, status: CiStatus) -> std::io::Result<()> {
        self.state()
            .filter(|state| state.matching(commit).is_some())
            .map_or(Ok(()), |_| self.save(&WatchState::new(commit.clone(), status)))
    }

    pub fn tracked(&self) -> Option<Commit> {
        self.state().map(|state| state.commit().clone())
    }

    pub fn status_of(&self, commit: &Commit) -> Option<CiStatus> {
        self.state().as_ref().and_then(|state| state.matching(commit)).map(WatchState::status)
    }

    pub fn status(&self) -> Option<CiStatus> {
        self.state().map(|state| state.status())
    }

    fn save(&self, state: &WatchState) -> std::io::Result<()> {
        write!(std::fs::File::create(self.0.join(Self::STATE))?, "{state}")
    }

    fn state(&self) -> Option<WatchState> {
        std::fs::read_to_string(self.0.join(Self::STATE)).ok().and_then(|text| text.parse().ok())
    }
}
