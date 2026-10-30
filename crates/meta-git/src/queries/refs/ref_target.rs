use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;
use crate::ref_name::RefName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTarget(RefName);

impl From<RefName> for RefTarget {
    fn from(name: RefName) -> RefTarget {
        RefTarget(name)
    }
}

impl Query for RefTarget {
    type View = Option<ObjectId>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<ObjectId>, GitFailure> {
        let missing = |error: git2::Error| (error.code() == git2::ErrorCode::NotFound).then_some(None).ok_or(error);
        Ok(repository.refname_to_id(self.0.as_str()).map(|oid| Some(ObjectId::from(oid))).or_else(missing)?)
    }
}
