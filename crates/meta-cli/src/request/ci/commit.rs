use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;

use meta_git::ObjectId;
use meta_schema::CommitId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit(String);

impl Commit {
    pub fn commit_id(&self) -> Option<CommitId> {
        self.0.parse().ok()
    }

    pub fn argument(&self) -> OsString {
        OsString::from(&self.0)
    }

    pub fn object_id(&self) -> Option<ObjectId> {
        self.0.parse().ok()
    }
}

impl FromStr for Commit {
    type Err = std::convert::Infallible;

    fn from_str(sha: &str) -> Result<Self, Self::Err> {
        Ok(Self(sha.split_whitespace().next().unwrap_or_default().to_owned()))
    }
}

impl From<ObjectId> for Commit {
    fn from(id: ObjectId) -> Self {
        Self(id.to_string())
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
