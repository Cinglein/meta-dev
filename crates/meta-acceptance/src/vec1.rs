mod empty_list;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::acceptance_failure::AcceptanceFailure;
use empty_list::EmptyList;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vec1<Item>(Vec<Item>);

impl<Item> Vec1<Item> {
    #[must_use]
    pub fn as_slice(&self) -> &[Item] {
        &self.0
    }
}

impl<Item> TryFrom<Vec<Item>> for Vec1<Item> {
    type Error = AcceptanceFailure;

    fn try_from(items: Vec<Item>) -> Result<Self, Self::Error> {
        EmptyList::refused(items).map(Self).map_err(|_| AcceptanceFailure::Empty)
    }
}

impl<Item: Serialize> Serialize for Vec1<Item> {
    fn serialize<Out: Serializer>(&self, serializer: Out) -> Result<Out::Ok, Out::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, Item: Deserialize<'de>> Deserialize<'de> for Vec1<Item> {
    fn deserialize<In: Deserializer<'de>>(deserializer: In) -> Result<Self, In::Error> {
        EmptyList::refused(Vec::deserialize(deserializer)?).map(Self).map_err(In::Error::custom)
    }
}
