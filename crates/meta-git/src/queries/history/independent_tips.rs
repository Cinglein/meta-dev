use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndependentTips(Vec<ObjectId>);

impl From<Vec<ObjectId>> for IndependentTips {
    fn from(commits: Vec<ObjectId>) -> IndependentTips {
        IndependentTips(commits)
    }
}

impl Query for IndependentTips {
    type View = Vec<ObjectId>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<ObjectId>, GitFailure> {
        let behind = |tip: &ObjectId| {
            self.0.iter().filter(|other| *other != tip).try_fold(false, |behind, other| {
                Ok::<bool, git2::Error>(behind || repository.graph_descendant_of(other.oid(), tip.oid())?)
            })
        };
        let independent = self.0.iter().map(|tip| behind(tip).map(|behind| (!behind).then_some(*tip)));
        Ok(independent.collect::<Result<Vec<Option<ObjectId>>, _>>()?.into_iter().flatten().collect())
    }
}
