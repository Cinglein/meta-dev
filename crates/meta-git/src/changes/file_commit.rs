use crate::change::Change;
use crate::files::Files;
use crate::git_failure::GitFailure;
use crate::identity::Identity;
use crate::object_id::ObjectId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileCommit {
    parent: Option<ObjectId>,
    files: Files,
    author: Identity,
    message: String,
}

impl FileCommit {
    #[must_use]
    pub fn new(parent: Option<ObjectId>, files: Files, author: Identity, message: String) -> FileCommit {
        FileCommit { parent, files, author, message }
    }
}

impl Change for FileCommit {
    type Applied = ObjectId;

    fn applied(&self, repository: &git2::Repository) -> Result<ObjectId, GitFailure> {
        let tree = repository.find_tree(self.files.tree(repository)?)?;
        let parents = self.parent.map(|parent| repository.find_commit(parent.oid())).transpose()?;
        let signature = self.author.signed_now()?;
        let parents: Vec<&git2::Commit<'_>> = parents.iter().collect();
        Ok(ObjectId::from(repository.commit(None, &signature, &signature, &self.message, &tree, &parents)?))
    }
}
