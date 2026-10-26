use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit(String);

impl Commit {
    pub fn object(&self, path: &Path) -> String {
        format!("{}:{}", self.0, path.display())
    }
}

impl FromStr for Commit {
    type Err = Failure;

    fn from_str(text: &str) -> Result<Commit, Failure> {
        let id = text.trim();
        (!id.is_empty() && id.chars().all(|digit| digit.is_ascii_hexdigit()))
            .then(|| Commit(id.to_owned()))
            .ok_or(Failure::NoDefaultBranch)
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
