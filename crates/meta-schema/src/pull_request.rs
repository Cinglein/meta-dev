use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::unrecognised::Unrecognised;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PullRequest(NonZeroU64);

impl FromStr for PullRequest {
    type Err = Unrecognised;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.strip_prefix('#')
            .unwrap_or(text)
            .parse()
            .map(Self)
            .map_err(|_| Unrecognised::new("a pull request number", text))
    }
}

impl fmt::Display for PullRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{}", self.0)
    }
}
