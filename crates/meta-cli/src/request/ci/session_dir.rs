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
    const STATES: &'static str = "states";
    const PARTIAL: &'static str = "partial";

    pub fn for_session(project: &Project, session: &SessionId) -> Option<Self> {
        GitLink::common_dir(project.root()).map(|base| Self(base.join(Self::ROOT).join(session.file_name())))
    }

    pub fn record(&self, line: impl fmt::Display) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        std::fs::OpenOptions::new().create(true).append(true).open(self.0.join(Self::EVENTS)).and_then(|events| {
            let mut whole = std::io::BufWriter::new(events);
            writeln!(whole, "{line}")?;
            whole.flush()
        })
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
        self.settle(commit, CiStatus::Running)
    }

    pub fn settle(&self, commit: &Commit, status: CiStatus) -> std::io::Result<()> {
        let folder = self.0.join(Self::STATES);
        std::fs::create_dir_all(&folder)?;
        let target = folder.join(commit.argument());
        let temp = target.with_extension(Self::PARTIAL);
        write!(std::fs::File::create(&temp)?, "{}", WatchState::new(commit.clone(), status))?;
        std::fs::rename(&temp, target)
    }

    pub fn status_of(&self, commit: &Commit) -> Option<CiStatus> {
        let text = std::fs::read_to_string(self.0.join(Self::STATES).join(commit.argument())).ok()?;
        let state: WatchState = text.parse().ok()?;
        state.matching(commit).map(WatchState::status)
    }

    pub fn watched(&self) -> Option<&Self> {
        std::fs::read_dir(self.0.join(Self::STATES)).ok()?.next().map(|_| self)
    }
}
