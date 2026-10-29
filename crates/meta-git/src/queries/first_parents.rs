use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstParents(ObjectId);

impl From<ObjectId> for FirstParents {
    fn from(commit: ObjectId) -> FirstParents {
        FirstParents(commit)
    }
}

impl Query for FirstParents {
    type View = Vec<ObjectId>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<ObjectId>, GitFailure> {
        let mut walk = repository.revwalk()?;
        walk.simplify_first_parent()?;
        walk.push(self.0.oid())?;
        Ok(walk.map(|oid| oid.map(ObjectId::from)).collect::<Result<_, _>>()?)
    }
}
