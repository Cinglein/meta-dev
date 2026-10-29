use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::marked_groups::MarkedGroups;
use super::restored::Restored;
use super::watch_item::WatchItem;
use crate::request::cli_failure::CliFailure;
use crate::request::local::LocalFile;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorState {
    marked: MarkedGroups,
    watched: Vec<WatchItem>,
}

impl MonitorState {
    const FILE: &'static str = "meta-local/monitor-state.json";
    const RESET: &'static str = "not valid monitor state; delete the state file to reset it";

    pub fn new(marked: MarkedGroups, watched: Vec<WatchItem>) -> Self {
        Self { marked, watched }
    }

    pub fn read(git_dir: &Path, _now: DateTime<Utc>) -> Result<Restored, CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        let text = file.text()?;
        let present = Some(text.trim()).filter(|text| !text.is_empty());
        present.map_or_else(
            || Ok(Restored::intact(Self::default())),
            |text| {
                serde_json::from_str(text)
                    .map(Restored::intact)
                    .map_err(|cause| file.failure(format!("{cause}; {}", Self::RESET)))
            },
        )
    }

    pub fn write(&self, git_dir: &Path) -> Result<(), CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        file.replaced(&serde_json::to_string(self).map_err(|failure| file.failure(failure))?)
    }

    pub fn marked(&self) -> &MarkedGroups {
        &self.marked
    }

    pub fn watched(&self, item: &WatchItem) -> Option<&WatchItem> {
        self.watched.iter().find(|before| item.same(before).is_some())
    }
}

impl fmt::Display for MonitorState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.watched.iter().try_for_each(|item| writeln!(formatter, "  {item}"))
    }
}
