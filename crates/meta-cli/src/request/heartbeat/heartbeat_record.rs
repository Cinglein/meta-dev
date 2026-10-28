use meta_door::MetaStore;
use serde::{Deserialize, Serialize};

use super::gc_hook::GcHook;
use super::snapshot::Snapshot;
use crate::request::cli_failure::CliFailure;
use crate::request::local::LocalFile;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatRecord {
    latest: Snapshot,
    previous: Option<Snapshot>,
}

impl HeartbeatRecord {
    const FILE: &'static str = "meta-local/heartbeat.json";

    pub fn read(store: &MetaStore) -> Result<Option<Self>, CliFailure> {
        let file = LocalFile::new(store.entry(Self::FILE));
        let text = file.text()?;
        (!text.is_empty()).then(|| serde_json::from_str(&text)).transpose().map_err(|failure| file.failure(failure))
    }

    pub fn succeeding(before: Option<Self>, latest: Snapshot) -> Self {
        Self { latest, previous: before.map(|before| before.latest) }
    }

    pub fn write(&self, store: &MetaStore) -> Result<(), CliFailure> {
        let file = LocalFile::new(store.entry(Self::FILE));
        file.replaced(&serde_json::to_string(self).map_err(|failure| file.failure(failure))?)
    }

    pub fn reported(self, gc: GcHook) -> Self {
        Self { latest: self.latest.reported(gc), ..self }
    }

    pub fn latest(&self) -> &Snapshot {
        &self.latest
    }

    #[cfg(test)]
    pub fn previous(&self) -> Option<&Snapshot> {
        self.previous.as_ref()
    }
}
