mod commit;

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::failure::Failure;
use commit::Commit;

#[derive(Clone, Debug)]
pub struct Revision {
    root: PathBuf,
    commit: Commit,
}

impl Revision {
    const DEFAULT_BRANCH: &str = "origin/HEAD";

    pub fn merge_base(root: &Path) -> Result<Revision, Failure> {
        let output = Self::git(root, &["merge-base", "HEAD", Self::DEFAULT_BRANCH])?;
        let commit = String::from_utf8_lossy(&output.stdout).parse().map_err(|_| Failure::NoDefaultBranch)?;
        Ok(Revision { root: root.to_path_buf(), commit })
    }

    pub fn file(&self, path: &Path) -> Result<Option<String>, Failure> {
        let object = self.commit.object(path);
        let output = Self::git(&self.root, &["cat-file", "blob", &object])?;
        Ok(output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned()))
    }

    pub fn changed_paths(&self) -> Result<Vec<String>, Failure> {
        let output = Self::command(&self.root)
            .args(["diff", "--name-only", "--no-renames", "-z"])
            .arg(&self.commit)
            .output()
            .map_err(Failure::Git)?;
        let paths = String::from_utf8_lossy(&output.stdout);
        let paths = paths.split('\0').filter(|path| !path.is_empty()).map(str::to_owned).collect();
        output
            .status
            .success()
            .then_some(paths)
            .ok_or_else(|| Failure::Git(io::Error::other(String::from_utf8_lossy(&output.stderr).into_owned())))
    }

    fn git(root: &Path, args: &[&str]) -> Result<Output, Failure> {
        Self::command(root).args(args).output().map_err(Failure::Git)
    }

    fn command(root: &Path) -> Command {
        let mut command = Command::new("git");
        command.arg("-C").arg(root);
        command
    }
}
