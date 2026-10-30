use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpushed {
    commits: Vec<ObjectId>,
    remote: RemoteName,
    held: Vec<ObjectId>,
}

impl Unpushed {
    #[must_use]
    pub fn new(commits: Vec<ObjectId>, remote: RemoteName, held: Vec<ObjectId>) -> Unpushed {
        Unpushed { commits, remote, held }
    }
}

impl Query for Unpushed {
    type View = Vec<ObjectId>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<ObjectId>, GitFailure> {
        let mut walk = repository.revwalk()?;
        self.commits.iter().for_each(|commit| drop(walk.push(commit.oid())));
        walk.hide_glob(&self.remote.tracking_glob())?;
        self.held.iter().for_each(|held| drop(walk.hide(held.oid())));
        Ok(walk.map(|oid| oid.map(ObjectId::from)).collect::<Result<_, _>>()?)
    }
}
