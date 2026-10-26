use std::path::Path;

use serde::{Deserialize, Serialize};

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

    pub fn read(git_dir: &Path) -> Result<Option<Self>, CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        let text = file.text()?;
        Some(text.trim())
            .filter(|text| !text.is_empty())
            .map(serde_json::from_str)
            .transpose()
            .map_err(|failure| file.failure(failure))
    }

    pub fn succeeding(before: Option<Self>, latest: Snapshot) -> Self {
        Self { latest, previous: before.map(|before| before.latest) }
    }

    pub fn write(&self, git_dir: &Path) -> Result<(), CliFailure> {
        let file = LocalFile::new(git_dir.join(Self::FILE));
        file.replaced(serde_json::to_string(self).map_err(|failure| file.failure(failure))?)
    }

    pub fn latest(&self) -> &Snapshot {
        &self.latest
    }

    #[cfg(test)]
    pub fn previous(&self) -> Option<&Snapshot> {
        self.previous.as_ref()
    }
}
