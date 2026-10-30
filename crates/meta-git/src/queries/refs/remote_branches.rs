use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;
use crate::ref_name::RefName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteBranches {
    Containing(ObjectId),
    At(ObjectId),
}

impl RemoteBranches {
    const REMOTES: &'static str = "refs/remotes/*";

    fn holds(self, repository: &git2::Repository, tip: git2::Oid) -> Result<bool, git2::Error> {
        match self {
            RemoteBranches::At(commit) => Ok(tip == commit.oid()),
            RemoteBranches::Containing(commit) => {
                Ok(tip == commit.oid() || repository.graph_descendant_of(tip, commit.oid())?)
            }
        }
    }
}

impl Query for RemoteBranches {
    type View = Vec<RefName>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<RefName>, GitFailure> {
        let branches = repository.references_glob(Self::REMOTES)?.map(|branch| {
            let branch = branch?;
            let tip = branch.resolve()?.target();
            let held = tip.map(|tip| self.holds(repository, tip)).transpose()?.unwrap_or(false);
            let Ok(name) = String::from_utf8_lossy(branch.name_bytes()).parse();
            Ok(held.then_some(name))
        });
        Ok(branches.collect::<Result<Vec<Option<RefName>>, git2::Error>>()?.into_iter().flatten().collect())
    }
}
