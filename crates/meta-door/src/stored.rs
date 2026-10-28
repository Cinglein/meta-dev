use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::entry::Entry;
use crate::git_dirs::GitDirs;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stored {
    common: PathBuf,
    path: PathBuf,
}

impl Stored {
    const AREAS: [&str; 2] = ["meta-local", "meta-ci"];

    pub(crate) fn placed(common: &Path, relative: &Path) -> Entry {
        let path = common.join(relative);
        if Self::area(relative).is_some()
            && relative.components().all(|component| matches!(component, Component::Normal(_)))
        {
            Entry::Stored(Stored { common: common.to_path_buf(), path })
        } else {
            Entry::Checked(path)
        }
    }

    pub(crate) fn moved(&self, moved: impl FnOnce(&Path) -> PathBuf) -> Entry {
        Self::placed(&self.common, &moved(&self.relative()))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn cleared(self) -> io::Result<PathBuf> {
        let area = Self::area(&self.relative()).unwrap_or_default().to_owned();
        GitDirs::within(self.path, &self.common, &area)
    }

    fn relative(&self) -> PathBuf {
        self.path.strip_prefix(&self.common).unwrap_or(&self.path).to_path_buf()
    }

    fn area(relative: &Path) -> Option<&OsStr> {
        relative
            .components()
            .next()
            .map(Component::as_os_str)
            .filter(|first| Self::AREAS.iter().any(|area| *first == OsStr::new(area)))
    }
}
