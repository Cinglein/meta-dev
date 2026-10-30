use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use meta_schema::Actor;
use serde::{Deserialize, Serialize};

use crate::policy::Surface;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct QueueSettings {
    hot: Vec<Surface>,
    members: NonZeroUsize,
    landers: Vec<Actor>,
}

impl Default for QueueSettings {
    fn default() -> QueueSettings {
        QueueSettings {
            hot: Vec::new(),
            members: NonZeroUsize::MIN,
            landers: Vec::new(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::QueueSettings;

    #[test]
    fn an_empty_queue_table_is_the_default_that_batches_nothing_together() {
        let read: QueueSettings = toml::from_str("").unwrap();
        assert_eq!(read, QueueSettings::default());
        assert!(!read.admits(&[()]));
    }
}
