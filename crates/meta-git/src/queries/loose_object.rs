use std::path::PathBuf;

use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LooseObject(ObjectId);

impl LooseObject {
    const OBJECTS: &'static str = "objects";
}

impl From<ObjectId> for LooseObject {
    fn from(object: ObjectId) -> LooseObject {
        LooseObject(object)
    }
}

impl Query for LooseObject {
    type View = PathBuf;

    fn answered(&self, repository: &git2::Repository) -> Result<PathBuf, GitFailure> {
        Ok(repository.commondir().join(Self::OBJECTS).join(self.0.fanned()))
    }
}
