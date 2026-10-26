use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit(String);

impl Commit {
    const OBJECTS: &'static str = "objects";
    const FAN_OUT_LEN: usize = 2;

    pub fn argument(&self) -> OsString {
        OsString::from(&self.0)
    }

    pub fn loose_object(&self) -> PathBuf {
        let (fan, rest) = self.0.split_at_checked(Self::FAN_OUT_LEN).unwrap_or_default();
        PathBuf::from(Self::OBJECTS).join(fan).join(rest)
    }
}

impl FromStr for Commit {
    type Err = std::convert::Infallible;

    fn from_str(sha: &str) -> Result<Self, Self::Err> {
        Ok(Self(sha.split_whitespace().next().unwrap_or_default().to_owned()))
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
