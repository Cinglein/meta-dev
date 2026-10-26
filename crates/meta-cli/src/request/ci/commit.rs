use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit(String);

impl Commit {
    pub fn argument(&self) -> OsString {
        OsString::from(&self.0)
    }
}

impl FromStr for Commit {
    type Err = std::convert::Infallible;

    fn from_str(sha: &str) -> Result<Self, Self::Err> {
        Ok(Self(sha.trim().to_owned()))
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
