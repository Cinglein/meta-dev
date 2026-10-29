use std::str::FromStr;

use super::unreadable::Unreadable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartTime(String);

impl FromStr for StartTime {
    type Err = Unreadable;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        (!text.is_empty()).then(|| Self(text.to_owned())).ok_or(Unreadable)
    }
}
