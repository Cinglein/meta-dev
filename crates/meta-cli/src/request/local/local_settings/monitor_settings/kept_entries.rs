use std::num::NonZeroUsize;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct KeptEntries(NonZeroUsize);

impl KeptEntries {
    pub fn newest<Entry>(self, entries: Vec<Entry>) -> Vec<Entry> {
        let dropped = entries.len().saturating_sub(self.0.get());
        entries.into_iter().skip(dropped).collect()
    }
}
