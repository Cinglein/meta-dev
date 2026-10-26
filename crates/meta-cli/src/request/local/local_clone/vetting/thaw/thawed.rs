use std::fs::Permissions;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thawed {
    path: PathBuf,
    permissions: Permissions,
}

impl Thawed {
    pub fn new(path: PathBuf, permissions: Permissions) -> Self {
        Self { path, permissions }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn permissions(&self) -> Permissions {
        self.permissions.clone()
    }
}
