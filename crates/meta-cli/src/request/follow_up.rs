use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FollowUp {
    words: Vec<String>,
    directory: PathBuf,
}

impl FollowUp {
    #[must_use]
    pub fn new(words: Vec<String>, directory: PathBuf) -> Self {
        Self { words, directory }
    }

    #[must_use]
    pub fn words(&self) -> &[String] {
        &self.words
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}
