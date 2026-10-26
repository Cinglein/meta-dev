use std::convert::Infallible;
use std::path::Path;
use std::str::FromStr;

use super::local_file::LocalFile;
use super::recorded_worktree::RecordedWorktree;
use super::trail_entry::TrailEntry;
use crate::request::cli_failure::CliFailure;

#[derive(Debug, Default)]
pub struct LocalTrail {
    entries: Vec<TrailEntry>,
    skipped: Vec<serde_json::Error>,
}

impl LocalTrail {
    const FILE: &'static str = "meta-local/worktrees.jsonl";

    pub fn read(git_dir: &Path) -> Result<Self, CliFailure> {
        let Ok(trail) = LocalFile::new(git_dir.join(Self::FILE)).text()?.parse();
        Ok(trail)
    }

    pub fn append(git_dir: &Path, entry: &TrailEntry) -> Result<(), CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        file.appended(serde_json::to_string(entry).map_err(|failure| file.failure(failure))?)
    }

    pub fn current(&self) -> Vec<&RecordedWorktree> {
        self.entries.iter().fold(Vec::new(), |current, entry| entry.applied(current))
    }

    pub fn skipped(self) -> Vec<serde_json::Error> {
        self.skipped
    }
}

impl FromStr for LocalTrail {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (entries, skipped): (Vec<_>, Vec<_>) =
            text.lines().map(serde_json::from_str::<TrailEntry>).partition(Result::is_ok);
        Ok(Self {
            entries: entries.into_iter().filter_map(Result::ok).collect(),
            skipped: skipped.into_iter().filter_map(Result::err).collect(),
        })
    }
}
