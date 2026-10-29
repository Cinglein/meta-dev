mod restored;

use std::fmt;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::fault::Fault;
use super::marked_groups::MarkedGroups;
use super::watch_item::WatchItem;
use crate::request::cli_failure::CliFailure;
use crate::request::local::LocalFile;
use restored::Restored;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorState {
    marked: MarkedGroups,
    watched: Vec<WatchItem>,
}

impl MonitorState {
    const FILE: &'static str = "meta-local/monitor-state.json";
    const STAMP: &'static str = "%Y%m%dT%H%M%SZ";

    pub fn new(marked: MarkedGroups, watched: Vec<WatchItem>) -> Self {
        Self { marked, watched }
    }

    pub fn read(git_dir: &Path, now: DateTime<Utc>) -> Result<Restored, CliFailure> {
        let path = git_dir.join(Self::FILE);
        let text = LocalFile::new(path.clone()).text()?;
        let present = Some(text.trim()).filter(|text| !text.is_empty());
        let parsed = present.map(serde_json::from_str::<Self>);
        Ok(match parsed {
            Some(Err(cause)) => Restored::recovered(Self::set_aside(&path, now, &cause)),
            Some(Ok(state)) => Restored::intact(state),
            None => Restored::intact(Self::default()),
        })
    }

    fn set_aside(path: &Path, now: DateTime<Utc>, cause: &serde_json::Error) -> Fault {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let aside = path.with_file_name(format!("{name}.{}.corrupt", now.format(Self::STAMP)));
        let kept = std::fs::rename(path, &aside).map_or_else(
            |failure| format!("could not keep it aside ({failure})"),
            |()| format!("kept as {}", aside.display()),
        );
        Fault::shown(format!("{} is not valid ({cause}); {kept}; patrolling from a fresh state", path.display()))
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
