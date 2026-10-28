use std::ffi::OsStr;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use crate::admin_file::AdminFile;
use crate::git_dirs::GitDirs;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminDir(PathBuf);

impl AdminDir {
    pub(crate) fn new(real: PathBuf) -> AdminDir {
        AdminDir(real)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn text(&self, file: AdminFile) -> io::Result<String> {
        fs::read_to_string(self.0.join(file.name()))
    }

    pub fn described(&self, path: &Path) -> io::Result<Metadata> {
        self.under(path)?.symlink_metadata()
    }

    pub fn listed(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(self.under(path)?)?.map(|entry| entry.map(|entry| path.join(entry.file_name()))).collect()
    }

    fn under(&self, path: &Path) -> io::Result<PathBuf> {
        GitDirs::within(path.to_path_buf(), &self.0, OsStr::new(""))
    }
}
