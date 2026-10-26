use std::path::Path;
use std::str::FromStr;

use super::local_file::LocalFile;
use super::recorded_worktree::RecordedWorktree;
use super::trail_entry::TrailEntry;
use crate::request::cli_failure::CliFailure;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct LocalTrail(Vec<TrailEntry>);

impl LocalTrail {
    const FILE: &'static str = "meta-local/worktrees.jsonl";

    pub fn read(git_dir: &Path) -> Result<Self, CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        file.text()?.parse().map_err(|failure| file.failure(failure))
    }

    pub fn append(git_dir: &Path, entry: &TrailEntry) -> Result<(), CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        file.appended(serde_json::to_string(entry).map_err(|failure| file.failure(failure))?)
    }

    pub fn current(&self) -> Vec<&RecordedWorktree> {
        self.0.iter().fold(Vec::new(), |current, entry| entry.applied(current))
    }
}

impl FromStr for LocalTrail {
    type Err = serde_json::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.lines().map(serde_json::from_str).collect::<Result<_, _>>().map(Self)
    }
}
