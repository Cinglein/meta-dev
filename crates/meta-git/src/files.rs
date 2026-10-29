use std::collections::BTreeMap;

use crate::git_failure::GitFailure;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Files(BTreeMap<String, Option<String>>);

impl Files {
    #[must_use]
    pub fn only(&self, name: &str) -> Option<&str> {
        self.0.get(name).filter(|_| self.0.len() == 1)?.as_deref()
    }

    pub(crate) fn tree(&self, repository: &git2::Repository) -> Result<git2::Oid, GitFailure> {
        let mut tree = repository.treebuilder(None)?;
        self.0.iter().try_for_each(|(name, text)| {
            let blob = repository.blob(text.as_deref().unwrap_or_default().as_bytes())?;
            tree.insert(name.as_str(), blob, git2::FileMode::Blob.into()).map(drop)
        })?;
        Ok(tree.write()?)
    }
}

impl FromIterator<(String, Option<String>)> for Files {
    fn from_iter<Entries: IntoIterator<Item = (String, Option<String>)>>(entries: Entries) -> Files {
        Files(entries.into_iter().collect())
    }
}
