use std::path::PathBuf;

use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeText {
    commit: ObjectId,
    path: PathBuf,
}

impl TreeText {
    #[must_use]
    pub fn new(commit: ObjectId, path: PathBuf) -> TreeText {
        TreeText { commit, path }
    }
}

impl Query for TreeText {
    type View = Option<String>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<String>, GitFailure> {
        let tree = repository.find_commit(self.commit.oid())?.tree()?;
        let missing = |error: git2::Error| (error.code() == git2::ErrorCode::NotFound).then_some(None).ok_or(error);
        let entry = tree.get_path(&self.path).map(Some).or_else(missing)?;
        let blob = entry.map(|entry| entry.to_object(repository)?.peel_to_blob()).transpose()?;
        Ok(blob.map(|blob| String::from_utf8_lossy(blob.content()).into_owned()))
    }
}
