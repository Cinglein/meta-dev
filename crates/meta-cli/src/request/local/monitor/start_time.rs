use std::str::FromStr;

use super::unreadable::Unreadable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartTime(String);

impl FromStr for StartTime {
    type Err = Unreadable;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = text.split_whitespace().collect();
        (!words.is_empty()).then(|| Self(words.join(" "))).ok_or(Unreadable)
    }
}
