use std::path::{Path, PathBuf};

use crate::change::Change;
use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorktreeRemove {
    Named(String),
    At(PathBuf),
}

impl WorktreeRemove {
    fn found(&self, repository: &git2::Repository) -> Result<git2::Worktree, git2::Error> {
        match self {
            WorktreeRemove::Named(name) => repository.find_worktree(name),
            WorktreeRemove::At(path) => Self::listed_at(repository, path),
        }
    }

    fn listed_at(repository: &git2::Repository, path: &Path) -> Result<git2::Worktree, git2::Error> {
        let wanted = path.canonicalize().ok();
        let names = repository.worktrees()?;
        let listed = names.iter().flatten().flatten().filter_map(|name| repository.find_worktree(name).ok());
        let found = listed.into_iter().find(|worktree| {
            worktree.path() == path || (wanted.is_some() && worktree.path().canonicalize().ok() == wanted)
        });
        found.ok_or_else(|| git2::Error::from_str(Self::UNLISTED))
    }

    const UNLISTED: &'static str = "not a linked worktree of this repository";
}

impl From<String> for WorktreeRemove {
    fn from(name: String) -> WorktreeRemove {
        WorktreeRemove::Named(name)
    }
}

impl Change for WorktreeRemove {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        let mut options = git2::WorktreePruneOptions::new();
        options.valid(true).working_tree(true);
        self.found(repository)?.prune(Some(&mut options))?;
        Ok(())
    }
}
