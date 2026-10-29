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
    use meta_laws::Laws;

    use super::{Text, TextFailure};

    const PADDED: [(&str, &str); 1] = [("  two words ", "two words")];
    const EMPTY: [&str; 2] = ["", " "];
    const MULTILINE: [&str; 1] = ["a\nb"];
    const STORED_PADDED: &str = "\" x\"";

    #[test]
    fn text_is_one_trimmed_non_empty_line_and_stored_text_is_already_trimmed() {
        let empty = Laws::misread::<Text>(&PADDED, &EMPTY, |_| TextFailure::Empty);
        let multiline = Laws::misread::<Text>(&[], &MULTILINE, |_| TextFailure::ControlCharacter);
        assert_eq!([empty, multiline].concat(), [""; 0]);
        let stored = serde_json::from_str::<Text>(STORED_PADDED).map_err(|failure| failure.to_string());
        assert!(stored.is_err_and(|failure| failure.contains(&TextFailure::Untrimmed.to_string())));
    }
}
