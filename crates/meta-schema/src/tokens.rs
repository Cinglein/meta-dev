use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::unrecognised::Unrecognised;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tokens(NonZeroU64);

impl FromStr for Tokens {
    type Err = Unrecognised;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse().map(Self).map_err(|_| Unrecognised::new("a positive token count", text))
    }
}

impl fmt::Display for Tokens {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} tokens", self.0)
    }
}
