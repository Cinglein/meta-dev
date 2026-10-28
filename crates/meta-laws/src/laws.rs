use std::fmt::Display;
use std::num::NonZeroU32;
use std::str::FromStr;

use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct Laws;

impl Laws {
    const POLICY: &str = include_str!("../../../tooling/policy.toml");
    const LIMITS: &str = "limits";
    const CASES: &str = "law_cases";

    #[must_use]
    pub fn cases() -> Option<NonZeroU32> {
        let policy: toml::Table = toml::from_str(Self::POLICY).ok()?;
        let cases = policy.get(Self::LIMITS)?.get(Self::CASES)?.as_integer()?;
        u32::try_from(cases).ok().and_then(NonZeroU32::new)
    }

    #[must_use]
    pub fn shown<Kind: FromStr + Display + PartialEq>(value: &Kind) -> bool {
        ToString::to_string(value).parse::<Kind>().is_ok_and(|read| read == *value)
    }

    #[must_use]
    pub fn settled<Kind: FromStr + Display + PartialEq>(text: &str) -> bool {
        text.parse::<Kind>().map_or(true, |value| Self::shown(&value))
    }

    #[must_use]
    pub fn stored<Kind: FromStr + Serialize + DeserializeOwned + PartialEq>(text: &str) -> bool {
        text.parse::<Kind>().is_ok_and(|value| Self::recorded(&value))
    }

    #[must_use]
    pub fn recorded<Kind: Serialize + DeserializeOwned + PartialEq>(value: &Kind) -> bool {
        let read = serde_json::to_string(value).and_then(|stored| serde_json::from_str::<Kind>(&stored));
        read.is_ok_and(|read| read == *value)
    }
}
