use std::path::PathBuf;

use crate::change::Change;
use crate::git_failure::GitFailure;
use crate::identity::Identity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeAdd {
    name: String,
    path: PathBuf,
    branch: String,
    identity: Identity,
}

impl WorktreeAdd {
    const CONFIG: &'static str = "config.worktree";

    #[must_use]
    pub fn new(name: String, path: PathBuf, branch: String, identity: Identity) -> WorktreeAdd {
        WorktreeAdd { name, path, branch, identity }
    }
}

impl Change for WorktreeAdd {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        self.path.parent().map_or(Ok(()), std::fs::create_dir_all)?;
        let branch = repository.find_branch(&self.branch, git2::BranchType::Local)?.into_reference();
        let mut options = git2::WorktreeAddOptions::new();
        options.reference(Some(&branch));
        let worktree = repository.worktree(&self.name, &self.path, Some(&options))?;
        let opened = git2::Repository::open_from_worktree(&worktree)?;
        self.identity.pinned(&opened.path().join(Self::CONFIG))
    }
}
