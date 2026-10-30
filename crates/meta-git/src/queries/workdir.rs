use std::path::PathBuf;

use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Workdir;

impl Query for Workdir {
    type View = Option<PathBuf>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<PathBuf>, GitFailure> {
        Ok(repository.workdir().map(PathBuf::from))
    }
}
