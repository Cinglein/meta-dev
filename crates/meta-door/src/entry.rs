use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::git_dirs::GitDirs;
use crate::stored::Stored;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Checked(PathBuf),
    Stored(Stored),
}

impl Entry {
    #[must_use]
    pub fn map(&self, moved: impl FnOnce(&Path) -> PathBuf) -> Entry {
        match self {
            Entry::Checked(path) => Entry::Checked(moved(path)),
            Entry::Stored(stored) => stored.moved(moved),
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Entry::Checked(path) => path,
            Entry::Stored(stored) => stored.path(),
        }
    }

    pub(crate) fn cleared(self) -> io::Result<PathBuf> {
        match self {
            Entry::Checked(path) => GitDirs::outside(path),
            Entry::Stored(stored) => stored.cleared(),
        }
    }
}

impl<P: AsRef<Path>> From<P> for Entry {
    fn from(path: P) -> Entry {
        Entry::Checked(path.as_ref().to_path_buf())
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.path().display())
    }
}
