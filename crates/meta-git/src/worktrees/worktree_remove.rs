use crate::change::Change;
use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeRemove(String);

impl From<String> for WorktreeRemove {
    fn from(name: String) -> WorktreeRemove {
        WorktreeRemove(name)
    }
}

impl Change for WorktreeRemove {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        let mut options = git2::WorktreePruneOptions::new();
        options.valid(true).working_tree(true);
        repository.find_worktree(&self.0)?.prune(Some(&mut options))?;
        Ok(())
    }
}
