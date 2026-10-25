use std::fmt;
use std::path::Path;
use std::str::FromStr;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit(String);

#[derive(Debug, Error)]
#[error("not a commit id: {0}")]
pub struct NotACommit(String);

impl Commit {
    pub fn object(&self, path: &Path) -> String {
        format!("{}:{}", self.0, path.display())
    }
}

impl FromStr for Commit {
    type Err = NotACommit;

    fn from_str(text: &str) -> Result<Commit, NotACommit> {
        let id = text.trim();
        if !id.is_empty() && id.chars().all(|digit| digit.is_ascii_hexdigit()) {
            Ok(Commit(id.to_owned()))
        } else {
            Err(NotACommit(id.to_owned()))
        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
