mod id_failure;
mod raw_id;

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::entity::Entity;
use crate::unparsed::Unparsed;
use id_failure::IdFailure;
use raw_id::RawId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id<Kind: Entity> {
    raw: RawId,
    kind: PhantomData<fn() -> Kind>,
}

impl<Kind: Entity> Id<Kind> {
    #[must_use]
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
            .ok_or_else(|| IdFailure::WrongKind { expected: Kind::PREFIX, found: Unparsed::from(text) })?;
        Ok(Self { raw: hex.parse()?, kind: PhantomData })
    }
}

impl<Kind: Entity> fmt::Display for Id<Kind> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{}", Kind::PREFIX, self.raw)
    }
}

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
    use meta_laws::{Laws, Reading};

    use super::{Id, IdFailure};
    use crate::entity::{Entity, Task};

    const ACCEPTED: [Reading; 1] = [Reading::verbatim("task-00000000000000a1")];
    const WRONG_KIND: [&str; 2] = ["task", "concern-0000000000000001"];
    const MALFORMED: [&str; 4] =
        ["task-123", "task-0123456789ABCDEF", "task-+123456789abcdef", "task-000000000000000g"];

    #[test]
    fn an_id_is_its_kind_then_sixteen_lowercase_hex_digits() {
        let wrong = |text: &str| IdFailure::WrongKind { expected: Task::PREFIX, found: text.into() };
        let misread = [
            Laws::misread::<Id<Task>>(&ACCEPTED, &WRONG_KIND, wrong),
            Laws::misread::<Id<Task>>(&[], &MALFORMED, |_| IdFailure::Malformed),
        ];
        assert_eq!(misread.concat(), Laws::NONE_MISREAD);
    }
}
