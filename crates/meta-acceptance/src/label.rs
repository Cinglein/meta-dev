use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::acceptance_failure::AcceptanceFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label(String);

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Label {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Label {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        let label = String::deserialize(deserializer)?;
        (!label.is_empty() && label.trim() == label && !label.chars().any(char::is_control))
            .then_some(Label(label))
            .ok_or_else(|| In::Error::custom(AcceptanceFailure::Blank))
    }
}
