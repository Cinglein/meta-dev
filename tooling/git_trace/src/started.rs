use std::collections::BTreeMap;
use std::fmt;

use task::Most;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Started<Key: Ord>(BTreeMap<Key, Most>);

impl<Key: Ord> Started<Key> {
    pub(crate) fn added(mut self, key: Key, count: Most) -> Started<Key> {
        let held = self.0.entry(key).or_insert(Most::from(0));
        *held = held.plus(count).unwrap_or(*held);
        self
    }

    pub(crate) fn joined(self, other: Started<Key>) -> Started<Key> {
        other.0.into_iter().fold(self, |joined, (key, count)| joined.added(key, count))
    }

    pub(crate) fn total(&self) -> Most {
        self.0.values().fold(Most::from(0), |total, count| total.plus(*count).unwrap_or(total))
    }

    pub(crate) fn counts(&self) -> &BTreeMap<Key, Most> {
        &self.0
    }
}

impl<Key: Ord> Default for Started<Key> {
    fn default() -> Started<Key> {
        Started(BTreeMap::new())
    }
}

impl<Key: Ord + fmt::Display> fmt::Display for Started<Key> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let counted = self.0.iter().filter(|(_, count)| count.count() > 0);
        let shown: Vec<String> = counted.map(|(key, count)| format!("{key} {count}")).collect();
        formatter.write_str(&shown.join(", "))
    }
}
