mod text_failure;

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use text_failure::TextFailure;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Text(String);

impl FromStr for Text {
    type Err = TextFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match (text.trim(), text.chars().any(char::is_control)) {
            (_, true) => Err(TextFailure::ControlCharacter),
            ("", false) => Err(TextFailure::Empty),
            (trimmed, false) => Ok(Self(trimmed.to_owned())),
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Text {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        let stored = String::deserialize(deserializer)?;
        stored
            .parse()
            .and_then(|text: Self| (text.0 == stored).then_some(text).ok_or(TextFailure::Untrimmed))
            .map_err(In::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Text;

    const PADDED: &str = "  two words ";
    const MULTILINE: &str = "a\nb";
    const STORED_PADDED: &str = "\" x\"";

    #[test]
    fn text_is_one_trimmed_non_empty_line() {
        assert_eq!(PADDED.parse::<Text>().map(|text| text.to_string()).ok(), Some(PADDED.trim().to_owned()));
        assert!(MULTILINE.parse::<Text>().is_err() && " ".parse::<Text>().is_err());
        assert!(serde_json::from_str::<Text>(STORED_PADDED).is_err());
    }
}
