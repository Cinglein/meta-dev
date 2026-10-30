use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergeBase(String);

impl MergeBase {
    const HEAD: &'static str = "HEAD";
}

impl From<&str> for MergeBase {
    fn from(base: &str) -> MergeBase {
        MergeBase(base.to_owned())
    }
}

impl Query for MergeBase {
    type View = ObjectId;

    fn answered(&self, repository: &git2::Repository) -> Result<ObjectId, GitFailure> {
        let commit = |spec: &str| repository.revparse_single(spec).and_then(|object| object.peel_to_commit());
        Ok(ObjectId::from(repository.merge_base(commit(Self::HEAD)?.id(), commit(&self.0)?.id())?))
    }
}
