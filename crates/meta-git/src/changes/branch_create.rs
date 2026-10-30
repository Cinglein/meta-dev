use crate::change::Change;
use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchCreate {
    name: String,
    at: ObjectId,
}

impl BranchCreate {
    #[must_use]
    pub fn new(name: String, at: ObjectId) -> BranchCreate {
        BranchCreate { name, at }
    }
}

impl Change for BranchCreate {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        repository.branch(&self.name, &repository.find_commit(self.at.oid())?, false)?;
        Ok(())
    }
}
