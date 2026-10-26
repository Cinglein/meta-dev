mod commit;

use std::path::{Path, PathBuf};

use task::{CommandLine, Program, Stdout};

use crate::task::failure::Failure;
use commit::Commit;

#[derive(Debug)]
pub struct Revision {
    root: PathBuf,
    commit: Commit,
}

impl Revision {
    const GIT: &str = "git";
    const DEFAULT_BRANCH: &str = "origin/HEAD";

    pub fn merge_base(root: &Path) -> Result<Revision, Failure> {
        let found = Self::git(root).arguments(["merge-base", "HEAD", Self::DEFAULT_BRANCH]).captured::<Stdout>();
        let commit = found.map_err(|_| Failure::NoDefaultBranch)?.parse()?;
        Ok(Revision { root: root.to_path_buf(), commit })
    }

    pub fn file(&self, path: &Path) -> Option<String> {
        Self::git(&self.root)
            .arguments(["cat-file", "blob"])
            .argument(self.commit.object(path))
            .captured::<Stdout>()
            .ok()
    }

    pub fn changed_paths(&self) -> Result<Vec<PathBuf>, Failure> {
        let listed = Self::git(&self.root)
            .arguments(["diff", "--name-only", "--no-renames", "-z"])
            .argument(&self.commit)
            .captured::<Stdout>()?;
        Ok(listed.split('\0').filter(|path| !path.is_empty()).map(PathBuf::from).collect())
    }

    fn git(root: &Path) -> CommandLine {
        CommandLine::new(Program::Executable(PathBuf::from(Self::GIT))).in_directory(root.to_path_buf())
    }
}
