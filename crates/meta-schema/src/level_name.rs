mod level_name_failure;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::unparsed::Unparsed;
pub use level_name_failure::LevelNameFailure;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LevelName(String);

impl FromStr for LevelName {
    type Err = LevelNameFailure;

    fn from_str(name: &str) -> Result<LevelName, LevelNameFailure> {
        let spelled = |letter: char| letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '-';
        (name.chars().any(|letter| letter.is_ascii_lowercase()) && name.chars().all(spelled))
            .then(|| LevelName(name.to_owned()))
            .ok_or_else(|| LevelNameFailure::Misspelled(Unparsed::from(name)))
    }
}

impl TryFrom<String> for LevelName {
    type Error = LevelNameFailure;

    fn try_from(name: String) -> Result<LevelName, LevelNameFailure> {
        name.parse()
    }
}

impl From<LevelName> for String {
    fn from(name: LevelName) -> String {
        name.0
    }
}

impl fmt::Display for LevelName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests;
