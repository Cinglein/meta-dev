use std::fmt;
use std::str::FromStr;

use super::BriefFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules(String);

impl Rules {
    pub const FILE: &'static str = "self/agent-rules.md";
    const VERSION_PREFIX: &'static str = "version: ";
}

impl FromStr for Rules {
    type Err = BriefFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.lines()
            .next()
            .and_then(|line| line.strip_prefix(Self::VERSION_PREFIX))
            .and_then(|version| version.parse::<u32>().ok())
            .map(|_| Self(text.to_owned()))
            .ok_or(BriefFailure::Unversioned)
    }
}

impl fmt::Display for Rules {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0.trim_end())
    }
}
