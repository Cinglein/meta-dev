use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::vocabulary_failure::VocabularyFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeasureName(String);

impl FromStr for MeasureName {
    type Err = VocabularyFailure;

    fn from_str(name: &str) -> Result<MeasureName, VocabularyFailure> {
        let spelled = |letter: char| letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '_';
        (name.chars().any(|letter| letter.is_ascii_lowercase()) && name.chars().all(spelled))
            .then(|| MeasureName(name.to_owned()))
            .ok_or(VocabularyFailure::MisspelledMeasure)
    }
}

impl fmt::Display for MeasureName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for MeasureName {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for MeasureName {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?.parse().map_err(In::Error::custom)
    }
}
