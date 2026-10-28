use std::io;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::entry::Entry;

#[derive(Debug)]
pub struct Scratch(TempDir);

impl Scratch {
    pub fn made(parent: impl Into<Entry>) -> io::Result<Scratch> {
        TempDir::new_in(parent.into().cleared()?).map(Scratch)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }

    #[must_use]
    pub fn kept(self) -> PathBuf {
        self.0.keep()
    }
}
