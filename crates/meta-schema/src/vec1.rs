use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::acceptance_failure::AcceptanceFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vec1<Item>(Vec<Item>);

impl<Item> Vec1<Item> {
    #[must_use]
    pub fn as_slice(&self) -> &[Item] {
        &self.0
    }
}

impl<Item: Serialize> Serialize for Vec1<Item> {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, Item: Deserialize<'de>> Deserialize<'de> for Vec1<Item> {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        let items = Vec::deserialize(deserializer)?;
        (!items.is_empty()).then_some(Self(items)).ok_or_else(|| In::Error::custom(AcceptanceFailure::Empty))
    }
}
