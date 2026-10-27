use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::unrecognised::Unrecognised;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitId(String);

impl CommitId {
    const SHA1_LEN: usize = 40;
    const SHA256_LEN: usize = 64;
}

impl FromStr for CommitId {
    type Err = Unrecognised;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Some(text)
            .filter(|text| [Self::SHA1_LEN, Self::SHA256_LEN].contains(&text.len()))
            .filter(|text| text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
            .map(|text| Self(text.to_owned()))
            .ok_or_else(|| Unrecognised::new("a full commit id", text))
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for CommitId {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for CommitId {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?.parse().map_err(In::Error::custom)
    }
}
