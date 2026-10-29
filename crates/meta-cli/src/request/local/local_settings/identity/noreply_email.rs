use std::fmt;
use std::str::FromStr;

use meta_schema::Email;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoreplyEmail(Email);

impl NoreplyEmail {
    const REFUSED: &'static str = "must be a GitHub noreply address";
}

impl FromStr for NoreplyEmail {
    type Err = &'static str;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        Some(Email::from(address)).filter(Email::noreply).map(Self).ok_or(Self::REFUSED)
    }
}

impl<'de> Deserialize<'de> for NoreplyEmail {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|refused| In::Error::custom(format!("identity.email {refused}")))
    }
}

impl fmt::Display for NoreplyEmail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
