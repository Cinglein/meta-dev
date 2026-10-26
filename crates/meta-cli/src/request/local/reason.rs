use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Reason(String);

impl FromStr for Reason {
    type Err = std::convert::Infallible;

    fn from_str(reason: &str) -> Result<Self, Self::Err> {
        Ok(Self(reason.to_owned()))
    }
}
