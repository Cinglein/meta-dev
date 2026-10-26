use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryNote {
    entry: Entry,
    text: Text,
}

impl EntryNote {
    #[must_use]
    pub fn new(entry: Entry, text: Text) -> Self {
        Self { entry, text }
    }

    #[must_use]
    pub fn entry(&self) -> Entry {
        self.entry
    }

    #[must_use]
    pub fn text(&self) -> &Text {
        &self.text
    }
}

impl fmt::Display for EntryNote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.entry, self.text)
    }
}
