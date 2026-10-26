mod entries_visitor;

use enum_map::{Enum, EnumMap};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use entries_visitor::EntriesVisitor;

#[derive(Debug)]
pub struct Entries<K: Enum, V>(EnumMap<K, Option<V>>);

impl<K: Enum + Copy, V> Entries<K, V> {
    pub fn entry(&self, key: K) -> Option<&V> {
        self.0[key].as_ref()
    }

    pub fn completed(self, filled: impl Fn(K) -> Option<V>) -> Result<EnumMap<K, V>, K> {
        self.0
            .try_map(|key, value| value.or_else(|| filled(key)).ok_or(key))
    }

    pub fn without(complete: &EnumMap<K, V>, moved: impl Fn(K) -> bool) -> Entries<K, V>
    where
        V: Clone,
    {
        Entries(EnumMap::from_fn(|key: K| {
            (!moved(key)).then(|| complete[key].clone())
        }))
    }
}

impl<K: Enum, V> From<EnumMap<K, Option<V>>> for Entries<K, V> {
    fn from(entries: EnumMap<K, Option<V>>) -> Entries<K, V> {
        Entries(entries)
    }
}

impl<K: Enum + Serialize, V: Serialize> Serialize for Entries<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.0
            .iter()
            .filter_map(|(key, value)| value.as_ref().map(|value| (key, value)))
            .try_for_each(|(key, value)| map.serialize_entry(&key, value))?;
        map.end()
    }
}

impl<'de, K: Enum + Deserialize<'de>, V: Deserialize<'de>> Deserialize<'de> for Entries<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Entries<K, V>, D::Error> {
        deserializer.deserialize_map(EntriesVisitor::default())
    }
}
