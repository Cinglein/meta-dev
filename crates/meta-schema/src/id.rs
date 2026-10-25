mod id_failure;
mod raw_id;

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::entity::Entity;
use id_failure::IdFailure;
use raw_id::RawId;

pub struct Id<Kind: Entity> {
    raw: RawId,
    kind: PhantomData<fn() -> Kind>,
}

impl<Kind: Entity> Id<Kind> {
    pub fn generate() -> Self {
        Self { raw: RawId::generate(), kind: PhantomData }
    }
}

impl<Kind: Entity> FromStr for Id<Kind> {
    type Err = IdFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let hex = text
            .strip_prefix(Kind::PREFIX)
            .and_then(|rest| rest.strip_prefix('-'))
            .ok_or_else(|| IdFailure::WrongKind { expected: Kind::PREFIX, found: text.to_owned() })?;
        Ok(Self { raw: hex.parse()?, kind: PhantomData })
    }
}

impl<Kind: Entity> fmt::Display for Id<Kind> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{}", Kind::PREFIX, self.raw)
    }
}

impl<Kind: Entity> fmt::Debug for Id<Kind> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl<Kind: Entity> Clone for Id<Kind> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Kind: Entity> Copy for Id<Kind> {}

impl<Kind: Entity> PartialEq for Id<Kind> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<Kind: Entity> Eq for Id<Kind> {}

impl<Kind: Entity> Serialize for Id<Kind> {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        serializer.collect_str(self)
    }
}

impl<'de, Kind: Entity> Deserialize<'de> for Id<Kind> {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        String::deserialize(deserializer)?.parse().map_err(In::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Id;
    use crate::entity::{Concern, Task};

    const MALFORMED: [&str; 4] = ["task", "task-123", "task-0123456789ABCDEF", "task-+123456789abcdef"];

    #[test]
    fn ids_round_trip_and_never_cross_kinds() {
        let task = Id::<Task>::generate();
        assert_eq!(task.to_string().parse::<Id<Task>>().ok(), Some(task));
        assert!(task.to_string().parse::<Id<Concern>>().is_err());
    }

    #[test]
    fn malformed_ids_are_rejected() {
        assert!(MALFORMED.iter().all(|text| text.parse::<Id<Task>>().is_err()));
    }
}
