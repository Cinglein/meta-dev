use std::ffi::OsStr;
use std::fmt;

use meta_schema::CommitId;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct HeadCommit(CommitId);

impl HeadCommit {
    #[must_use]
    pub fn commit_id(&self) -> &CommitId {
        &self.0
    }
}

impl TryFrom<String> for HeadCommit {
    type Error = &'static str;

    fn try_from(sha: String) -> Result<Self, Self::Error> {
        sha.parse().map(Self).map_err(|_| "a head commit that is not a full commit id")
    }
}

impl AsRef<OsStr> for HeadCommit {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl fmt::Display for HeadCommit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
