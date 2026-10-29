use crate::change::Change;
use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::ref_name::RefName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefMove {
    name: RefName,
    to: ObjectId,
}

impl RefMove {
    const REASON: &'static str = "meta";

    #[must_use]
    pub fn new(name: RefName, to: ObjectId) -> RefMove {
        RefMove { name, to }
    }
}

impl Change for RefMove {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        repository.reference(self.name.as_str(), self.to.oid(), true, Self::REASON)?;
        Ok(())
    }
}
