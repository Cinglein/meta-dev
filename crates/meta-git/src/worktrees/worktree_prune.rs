use crate::change::Change;
use crate::git_failure::GitFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorktreePrune;

impl Change for WorktreePrune {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        let names = repository.worktrees()?;
        let found = names.iter().flatten().flatten().filter_map(|name| repository.find_worktree(name).ok());
        found.filter(|worktree| worktree.is_prunable(None).unwrap_or_default()).try_for_each(|worktree| {
            worktree.prune(None)?;
            Ok(())
        })
    }
}
