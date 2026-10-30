use std::ffi::OsStr;
use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct HeadCommit(String);

impl TryFrom<String> for HeadCommit {
    type Error = &'static str;

    fn try_from(sha: String) -> Result<Self, Self::Error> {
        (!sha.trim().is_empty()).then_some(Self(sha)).ok_or("an empty head commit")
    }
}

impl AsRef<OsStr> for HeadCommit {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl fmt::Display for HeadCommit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
