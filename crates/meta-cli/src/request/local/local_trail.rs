mod torn_line;

use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::str::FromStr;

use meta_door::MetaStore;

use super::local_file::LocalFile;
use super::recorded_worktree::RecordedWorktree;
use super::trail_entry::TrailEntry;
use crate::request::cli_failure::CliFailure;
pub use torn_line::TornLine;

#[derive(Debug, Default)]
pub struct LocalTrail {
    entries: Vec<TrailEntry>,
    torn: Vec<TornLine>,
}

impl LocalTrail {
    const FILE: &'static str = "meta-local/worktrees.jsonl";

    pub fn read(store: &MetaStore) -> Result<Self, CliFailure> {
        let Ok(trail) = LocalFile::new(store.entry(Self::FILE)).text()?.parse();
        Ok(trail)
    }

    pub fn append(store: &MetaStore, entry: &TrailEntry) -> Result<(), CliFailure> {
        let file = LocalFile::new(store.entry(Self::FILE));
        file.appended(serde_json::to_string(entry).map_err(|failure| file.failure(failure))?)
    }

    pub fn current(&self) -> Vec<RecordedWorktree> {
        self.entries.iter().fold(Vec::new(), |current, entry| entry.applied(current))
    }

    pub fn torn(&self) -> &[TornLine] {
        &self.torn
    }
}

impl FromStr for LocalTrail {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (entries, torn): (Vec<_>, Vec<_>) = text
            .lines()
            .zip(std::iter::successors(Some(NonZeroUsize::MIN), |number| number.checked_add(1)))
            .map(|(line, number)| {
                serde_json::from_str::<TrailEntry>(line).map_err(|reason| TornLine::new(number, reason))
            })
            .partition(Result::is_ok);
        Ok(Self {
            entries: entries.into_iter().filter_map(Result::ok).collect(),
            torn: torn.into_iter().filter_map(Result::err).collect(),
        })
    }
}
