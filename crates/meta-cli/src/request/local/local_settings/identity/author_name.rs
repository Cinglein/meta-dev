use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct AuthorName(String);

impl FromStr for AuthorName {
    type Err = &'static str;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Some(name).filter(|name| !name.is_empty()).map(|name| Self(name.to_owned())).ok_or("is empty")
    }
}

impl fmt::Display for AuthorName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
