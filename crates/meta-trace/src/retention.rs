use std::io;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Retention(NonZeroUsize);

impl Retention {
    const KEPT: NonZeroUsize = match NonZeroUsize::new(20) {
        Some(kept) => kept,
        None => NonZeroUsize::MIN,
    };

    pub fn pruned(self, oldest_first: &[PathBuf]) -> io::Result<()> {
        let dropped = oldest_first.len().saturating_sub(self.0.get());
        oldest_first.iter().take(dropped).try_for_each(std::fs::remove_file)
    }
}

impl Default for Retention {
    fn default() -> Self {
        Self(Self::KEPT)
    }
}
