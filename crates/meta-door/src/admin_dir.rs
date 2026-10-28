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

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::Door;

    const COMMITTED: [&str; 9] =
        ["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "x"];
    const PACKED: [&str; 2] = ["pack-refs", "--all"];
    const INIT: [&str; 2] = ["init", "-q"];
    const ENTRY: &str = ".git";
    const HEADS: &str = "refs/heads";
    const REFS: [&str; 4] = ["refs", "packed-refs", "logs", "reftable"];

    #[test]
    fn an_admin_dir_never_lists_or_describes_refs_even_after_they_are_packed() {
        let repository = tempfile::tempdir().unwrap();
        let git = |words: &[&str]| Command::new("git").current_dir(repository.path()).args(words).status().unwrap();
        assert!([&INIT[..], &COMMITTED[..], &PACKED[..]].iter().all(|words| git(words).success()));
        let git_dir = repository.path().join(ENTRY);
        let admin = Door::inspected(&git_dir).unwrap();
        let heads = admin.listed(&git_dir.join(HEADS));
        let described = REFS.iter().any(|name| admin.described(&git_dir.join(name)).is_ok());
        assert!(heads.is_err() && !described);
    }
}
