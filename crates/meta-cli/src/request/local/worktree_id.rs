use std::ffi::OsStr;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorktreeId(String);

impl WorktreeId {
    pub fn parsed(git_dir: &Path) -> Option<Self> {
        git_dir.file_name().and_then(OsStr::to_str).map(|name| Self(name.to_owned()))
    }

    pub fn names(&self, git_dir: &Path) -> bool {
        git_dir.file_name() == Some(OsStr::new(&self.0))
    }
}
