use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use crate::entry::Entry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stored {
    common: PathBuf,
    path: PathBuf,
}

impl Stored {
    const AREAS: [&str; 2] = ["meta-local", "meta-ci"];

    pub(crate) fn placed(common: &Path, relative: &Path) -> Entry {
        let path = common.join(relative);
        if relative
            .components()
            .next()
            .is_some_and(|first| Self::AREAS.iter().any(|area| first.as_os_str() == OsStr::new(area)))
            && relative.components().all(|component| matches!(component, Component::Normal(_)))
        {
            Entry::Stored(Stored { common: common.to_path_buf(), path })
        } else {
            Entry::Checked(path)
        }
    }

    pub(crate) fn moved(&self, moved: impl FnOnce(&Path) -> PathBuf) -> Entry {
        Self::placed(&self.common, &moved(self.path.strip_prefix(&self.common).unwrap_or(&self.path)))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn into_path(self) -> PathBuf {
        self.path
    }
}
