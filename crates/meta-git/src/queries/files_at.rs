use crate::files::Files;
use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilesAt(ObjectId);

impl FilesAt {
    fn files(repository: &git2::Repository, commit: &git2::Commit<'_>) -> Result<Files, GitFailure> {
        let text = |entry: &git2::TreeEntry<'_>| {
            let blob = entry.to_object(repository)?.into_blob().ok();
            let name = String::from_utf8_lossy(entry.name_bytes()).into_owned();
            let text = blob.map(|blob| String::from_utf8(blob.content().to_vec())).transpose();
            Ok((name.clone(), text.map_err(|_| GitFailure::NotText(name))?))
        };
        commit.tree()?.iter().map(|entry| text(&entry)).collect()
    }
}

impl From<ObjectId> for FilesAt {
    fn from(commit: ObjectId) -> FilesAt {
        FilesAt(commit)
    }
}

impl Query for FilesAt {
    type View = Option<Files>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<Files>, GitFailure> {
        let missing = |error: git2::Error| (error.code() == git2::ErrorCode::NotFound).then_some(None).ok_or(error);
        let commit = repository.find_commit(self.0.oid()).map(Some).or_else(missing)?;
        commit.map(|commit| Self::files(repository, &commit)).transpose()
    }
}
