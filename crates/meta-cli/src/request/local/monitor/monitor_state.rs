use std::fmt;

use meta_door::MetaStore;
use serde::{Deserialize, Serialize};

use super::marked_groups::MarkedGroups;
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

    pub fn new(marked: MarkedGroups, watched: Vec<WatchItem>) -> Self {
        Self { marked, watched }
    }

    pub fn read(store: &MetaStore) -> Self {
        let text = LocalFile::new(store.entry(Self::FILE)).text().unwrap_or_default();
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn write(&self, store: &MetaStore) -> Result<(), CliFailure> {
        let file = LocalFile::new(store.entry(Self::FILE));
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
