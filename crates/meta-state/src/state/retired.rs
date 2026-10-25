use std::fmt;

use meta_schema::EntryNote;

use crate::stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retired {
    note: EntryNote,
    stamp: Stamp,
}

impl Retired {
    pub fn new(note: EntryNote, stamp: Stamp) -> Self {
        Self { note, stamp }
    }

    pub fn note(&self) -> &EntryNote {
        &self.note
    }
}

impl fmt::Display for Retired {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, " [retired by {}: {}]", self.stamp, self.note.text())
    }
}
