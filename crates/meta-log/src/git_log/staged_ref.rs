use meta_git::{ObjectId, Reads, RefMove, RefName, RefTarget, Repository, Writes};

use super::sha::Sha;
use crate::log_failure::LogFailure;

#[derive(Debug)]
pub struct StagedRef {
    repository: Repository,
    decided_on: Option<ObjectId>,
}

impl StagedRef {
    pub const NAME: &'static str = "refs/meta-log/staged";

    pub fn read(repository: Repository) -> Result<Self, LogFailure> {
        let decided_on = repository.read(&RefTarget::from(Self::name()))?;
        Ok(Self { repository, decided_on })
    }

    pub fn moved(&self, commit: Sha) -> Result<Option<Sha>, LogFailure> {
        let moved = self.repository.write(&RefMove::new(Self::name(), self.decided_on, commit.object_id()?));
        Ok(moved.map(|()| Some(commit)).or_else(|failure| failure.ref_moved().map(|_| None))?)
    }

    fn name() -> RefName {
        let Ok(name) = Self::NAME.parse();
        name
    }
}
