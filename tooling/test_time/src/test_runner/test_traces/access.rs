use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Access {
    Sealed,
    Open,
}

impl Access {
    pub(super) fn applied(self, directory: &Path) -> io::Result<()> {
        let mut permissions = fs::metadata(directory)?.permissions();
        permissions.set_readonly(self == Access::Sealed);
        fs::set_permissions(directory, permissions)
    }
}
