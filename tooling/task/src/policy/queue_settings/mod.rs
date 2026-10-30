use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use meta_schema::Actor;
use serde::{Deserialize, Serialize};

use crate::policy::Surface;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueSettings {
    hot: Vec<Surface>,
    members: NonZeroUsize,
    #[serde(default)]
    landers: Vec<Actor>,
}

impl QueueSettings {
    #[must_use]
    pub fn hot_among(&self, paths: &BTreeSet<PathBuf>) -> BTreeSet<Surface> {
        self.hot
            .iter()
            .filter(|hot| paths.iter().any(|path| hot.covers(path)))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn admits<T>(&self, members: &[T]) -> bool {
        members.len() < self.members.get()
    }
}
