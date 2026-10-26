use std::fmt;
use std::iter;
use std::marker::PhantomData;

use enum_map::{Enum, EnumMap};
use serde::Deserialize;
use serde::de::{MapAccess, Visitor};

use super::Entries;

pub struct EntriesVisitor<K, V> {
    keys: PhantomData<K>,
    values: PhantomData<V>,
}

impl<K, V> Default for EntriesVisitor<K, V> {
    fn default() -> EntriesVisitor<K, V> {
        EntriesVisitor {
            keys: PhantomData,
            values: PhantomData,
        }
    }
}

impl<'de, K: Enum + Deserialize<'de>, V: Deserialize<'de>> Visitor<'de> for EntriesVisitor<K, V> {
    type Value = Entries<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a table of policy entries")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Entries<K, V>, M::Error> {
        iter::from_fn(|| access.next_entry::<K, V>().transpose())
            .map(|entry| entry.map(|(key, value)| (key, Some(value))))
            .collect::<Result<EnumMap<K, Option<V>>, M::Error>>()
            .map(Entries::from)
    }
}
