use std::fs::{self, Permissions};
use std::io;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Dir,
    DirAll,
    RemoveFile,
    RemoveDir,
    RemoveDirAll,
    Permissions(Permissions),
}

impl Change {
    pub(crate) fn applied(self, path: &Path) -> io::Result<()> {
        match self {
            Change::Dir => fs::create_dir(path),
            Change::DirAll => fs::create_dir_all(path),
            Change::RemoveFile => fs::remove_file(path),
            Change::RemoveDir => fs::remove_dir(path),
            Change::RemoveDirAll => fs::remove_dir_all(path),
            Change::Permissions(permissions) => fs::set_permissions(path, permissions),
        }
    }
}
