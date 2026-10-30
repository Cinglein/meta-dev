use std::collections::BTreeMap;

use serde::Deserialize;

use super::flag::Flag;
use super::program::Program;
use super::union::Union;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct Table<Word>(BTreeMap<String, Vec<Word>>);

impl<Word> Table<Word> {
    pub fn words_for(&self, key: &str) -> &[Word] {
        self.0.get(key).map_or(&[], Vec::as_slice)
    }
}

impl Table<Flag> {
    pub fn command_in(&self, segment: &[String]) -> Option<String> {
        segment.iter().enumerate().find_map(|(index, word)| {
            let flags = self.words_for(Program::base(word));
            segment.get(index + 1..).and_then(|args| Flag::command_after(flags, args))
        })
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
