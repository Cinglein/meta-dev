use std::collections::BTreeMap;

use serde::Deserialize;

use super::union::Union;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct Table<Word>(BTreeMap<String, Vec<Word>>);

impl<Word> Table<Word> {
    pub fn words_for(&self, key: &str) -> &[Word] {
        self.0.get(key).map_or(&[], Vec::as_slice)
    }
}

impl<Word> Default for Table<Word> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<Word> Union for Table<Word> {
    fn union(self, other: Self) -> Self {
        let merge = |mut merged: BTreeMap<String, Vec<Word>>, (key, words): (String, Vec<Word>)| {
            merged.entry(key).or_default().extend(words);
            merged
        };
        Self(other.0.into_iter().fold(self.0, merge))
    }
}
