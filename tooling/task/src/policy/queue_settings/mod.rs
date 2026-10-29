mod hot_path;

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use hot_path::HotPath;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueSettings {
    hot: Vec<HotPath>,
    members: NonZeroUsize,
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
    pub fn admits<T>(&self, members: &[T]) -> bool {
        members.len() < self.members.get()
    }
}
