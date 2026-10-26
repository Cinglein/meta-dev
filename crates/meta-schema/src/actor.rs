mod actor_failure;

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::unparsed::Unparsed;
use actor_failure::ActorFailure;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Actor(String);

impl Actor {
    const OWNER: &'static str = "owner";

    #[must_use]
    pub fn owner() -> Self {
        Self(Self::OWNER.to_owned())
    }
}

impl FromStr for Actor {
    type Err = ActorFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Some(name)
            .filter(|name| name.bytes().next().is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
            .filter(|name| {
                name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_' | b'.'))
            })
            .map(|name| Self(name.to_owned()))
            .ok_or_else(|| ActorFailure::NotASlug(Unparsed::from(name)))
    }
}

impl fmt::Display for Actor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Actor {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Actor {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?.parse().map_err(In::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Actor;

    const GOOD: [&str; 3] = ["owner", "claude-1a2b", "a.b_c"];
    const BAD: [&str; 4] = ["", "Owner", "-x", "a/b"];

    #[test]
    fn actor_names_are_lowercase_slugs() {
        assert!(GOOD.iter().all(|name| name.parse::<Actor>().is_ok()));
        assert!(BAD.iter().all(|name| name.parse::<Actor>().is_err()));
    }
}
