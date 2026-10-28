use meta_door::kinds::{Permissions, PermissionsExt as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

mod mode;

use mode::Mode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thawed {
    path: PathBuf,
    mode: Mode,
}

impl Thawed {
    pub fn new(path: PathBuf, permissions: &Permissions) -> Self {
        Self { path, mode: Mode::new(permissions.mode()) }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn permissions(&self) -> Permissions {
        Permissions::from_mode(self.mode.bits())
    }
}
