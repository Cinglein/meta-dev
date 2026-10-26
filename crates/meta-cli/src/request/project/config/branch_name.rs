use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct BranchName(String);

impl BranchName {
    const LOG: &'static str = "meta/log";

    pub fn log(&self) -> Option<&Self> {
        self.0.starts_with(Self::LOG).then_some(self)
    }
}

impl FromStr for BranchName {
    type Err = std::convert::Infallible;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(Self(name.strip_prefix("refs/heads/").unwrap_or(name).to_owned()))
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
