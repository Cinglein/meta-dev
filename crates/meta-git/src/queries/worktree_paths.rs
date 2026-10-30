use std::path::{Path, PathBuf};

use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorktreePaths;

impl WorktreePaths {
    const GIT_DIR: &'static str = ".git";
}

impl Query for WorktreePaths {
    type View = Vec<PathBuf>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<PathBuf>, GitFailure> {
        let common = repository.commondir();
        let main = common.ends_with(Self::GIT_DIR).then(|| common.parent()).flatten().unwrap_or(common);
        let names = repository.worktrees()?;
        let linked = names.iter().flatten().flatten().map(|name| repository.find_worktree(name));
        let linked = linked.map(|worktree| worktree.map(|worktree| worktree.path().to_path_buf()));
        [Ok(Path::to_path_buf(main))]
            .into_iter()
            .chain(linked)
            .collect::<Result<_, git2::Error>>()
            .map_err(GitFailure::from)
    }
}
