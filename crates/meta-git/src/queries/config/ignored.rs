use std::path::PathBuf;

use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ignored(PathBuf);

impl From<PathBuf> for Ignored {
    fn from(path: PathBuf) -> Ignored {
        Ignored(path)
    }
}

impl Query for Ignored {
    type View = bool;

    fn answered(&self, repository: &git2::Repository) -> Result<bool, GitFailure> {
        Ok(repository.is_path_ignored(&self.0)?)
    }
}
