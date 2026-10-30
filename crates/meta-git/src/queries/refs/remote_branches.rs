use std::collections::{BTreeMap, HashSet};

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

    fn tips(repository: &git2::Repository) -> Result<BTreeMap<RefName, git2::Oid>, git2::Error> {
        repository.references_glob(Self::REMOTES)?.try_fold(BTreeMap::new(), |mut tips, branch| {
            let branch = branch?;
            let Ok(name) = String::from_utf8_lossy(branch.name_bytes()).parse();
            tips.extend(branch.resolve()?.target().map(|tip| (name, tip)));
            Ok(tips)
        })
    }

    fn reaching(
        repository: &git2::Repository,
        commit: git2::Oid,
        tips: impl Iterator<Item = git2::Oid>,
    ) -> Result<HashSet<git2::Oid>, git2::Error> {
        let mut walk = repository.revwalk()?;
        walk.set_sorting(git2::Sort::TOPOLOGICAL.union(git2::Sort::REVERSE))?;
        tips.into_iter().try_for_each(|tip| walk.push(tip))?;
        repository.find_commit(commit)?.parent_ids().try_for_each(|parent| walk.hide(parent))?;
        walk.try_fold(HashSet::new(), |mut reaching, id| {
            let id = id?;
            let found = repository.find_commit(id)?;
            let mut parents = found.parent_ids();
            reaching.extend((id == commit || parents.any(|parent| reaching.contains(&parent))).then_some(id));
            Ok(reaching)
        })
    }
}

impl Query for RemoteBranches {
    type View = Vec<RefName>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<RefName>, GitFailure> {
        let (RemoteBranches::Containing(commit) | RemoteBranches::At(commit)) = *self;
        repository.find_commit(commit.oid())?;
        let tips = Self::tips(repository)?;
        let held = match self {
            RemoteBranches::At(_) => [commit.oid()].into_iter().collect(),
            RemoteBranches::Containing(_) => Self::reaching(repository, commit.oid(), tips.values().copied())?,
        };
        Ok(tips.into_iter().filter(|(_, tip)| held.contains(tip)).map(|(name, _)| name).collect())
    }
}
