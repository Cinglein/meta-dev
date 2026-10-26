use std::fmt;

use meta_schema::Email;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoreplyEmail(Email);

impl<'de> Deserialize<'de> for NoreplyEmail {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        let email = Email::from(String::deserialize(deserializer)?.as_str());
        Some(email)
            .filter(Email::noreply)
            .map(Self)
            .ok_or_else(|| In::Error::custom("identity.email must be a GitHub noreply address"))
    }
}

impl fmt::Display for NoreplyEmail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
