use std::io;
use std::path::{Path, PathBuf};

use crate::admin_dir::AdminDir;
use crate::entry::Entry;
use crate::git_dirs::GitDirs;
use crate::stored::Stored;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaStore(PathBuf);

impl MetaStore {
    pub fn located(directory: &Path) -> io::Result<MetaStore> {
        GitDirs::common(directory).map(MetaStore)
    }

    #[must_use]
    pub fn common(&self) -> &Path {
        &self.0
    }

    #[must_use]
    pub fn admin(&self) -> AdminDir {
        AdminDir::new(self.0.clone())
    }

    #[must_use]
    pub fn entry(&self, relative: impl AsRef<Path>) -> Entry {
        Stored::placed(&self.0, relative.as_ref())
    }
}
