use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tag {
    AcceptanceCandidate,
}

#[derive(Debug, Error)]
pub enum TagFailure {
    #[error("unknown tag `{0}`; known: acceptance-candidate")]
    Unknown(String),
}

impl Tag {
    const ACCEPTANCE_CANDIDATE: &'static str = "acceptance-candidate";
}

impl FromStr for Tag {
    type Err = TagFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            Self::ACCEPTANCE_CANDIDATE => Ok(Self::AcceptanceCandidate),
            other => Err(TagFailure::Unknown(other.to_owned())),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AcceptanceCandidate => formatter.write_str(Self::ACCEPTANCE_CANDIDATE),
        }
    }
}

impl Serialize for Tag {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?.parse().map_err(In::Error::custom)
    }
}
