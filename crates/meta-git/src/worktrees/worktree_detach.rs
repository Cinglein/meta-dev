use std::path::PathBuf;

use crate::change::Change;
use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeDetach {
    path: PathBuf,
    commit: ObjectId,
}

impl WorktreeDetach {
    const PREFIX: &'static str = "detached-";
    const HIDDEN: char = '.';
    const UNNAMED: &'static str = "a worktree path needs a final name";

    #[must_use]
    pub fn new(path: PathBuf, commit: ObjectId) -> WorktreeDetach {
        WorktreeDetach { path, commit }
    }

    fn name(&self) -> Result<String, git2::Error> {
        let last = self.path.file_name().and_then(|name| name.to_str());
        let last = last.map(|name| name.trim_start_matches(Self::HIDDEN)).filter(|name| !name.is_empty());
        last.map(|name| [Self::PREFIX, name].concat()).ok_or_else(|| git2::Error::from_str(Self::UNNAMED))
    }

    fn detached(
        &self,
        repository: &git2::Repository,
        name: &str,
        commit: &git2::Commit<'_>,
    ) -> Result<(), git2::Error> {
        let branch = repository.branch(name, commit, false)?;
        let mut options = git2::WorktreeAddOptions::new();
        options.reference(Some(branch.get()));
        let added = repository.worktree(name, &self.path, Some(&options));
        added.and_then(|worktree| git2::Repository::open_from_worktree(&worktree)?.set_head_detached(commit.id()))
    }
}

impl Change for WorktreeDetach {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        let commit = repository.find_commit(self.commit.oid())?;
        let name = self.name()?;
        let empty =
            std::fs::read_dir(&self.path).ok().and_then(|mut entries| entries.next().is_none().then_some(&self.path));
        empty.map(std::fs::remove_dir).transpose()?;
        let detached = self.detached(repository, &name, &commit);
        let deleted = repository.find_branch(&name, git2::BranchType::Local).and_then(|mut branch| branch.delete());
        Ok(detached.and(deleted)?)
    }
}
