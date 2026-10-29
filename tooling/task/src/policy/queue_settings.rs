use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::batch_size::BatchSize;
use super::hot_path::HotPath;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueSettings {
    hot: Vec<HotPath>,
    members: BatchSize,
}

impl QueueSettings {
    #[must_use]
    pub fn hot_among<'paths>(
        &self,
        paths: impl IntoIterator<Item = &'paths PathBuf>,
    ) -> BTreeSet<PathBuf> {
        paths
            .into_iter()
            .flat_map(|path| self.hot.iter().filter_map(|hot| hot.meeting(path)))
            .collect()
    }

    #[must_use]
    pub fn members(&self) -> BatchSize {
        self.members
    }
}
