use crate::change::Change;
use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchDelete(String);

impl From<String> for BranchDelete {
    fn from(name: String) -> BranchDelete {
        BranchDelete(name)
    }
}

impl Change for BranchDelete {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        repository.find_branch(&self.0, git2::BranchType::Local)?.delete()?;
        Ok(())
    }
}
