use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision(String);

impl From<String> for Revision {
    fn from(spec: String) -> Revision {
        Revision(spec)
    }
}

impl Query for Revision {
    type View = Option<ObjectId>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<ObjectId>, GitFailure> {
        let commit = repository.revparse_single(&self.0).and_then(|object| object.peel_to_commit());
        Ok(commit.ok().map(|commit| ObjectId::from(commit.id())))
    }
}
