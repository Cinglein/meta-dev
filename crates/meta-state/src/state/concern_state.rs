use std::fmt;

use meta_schema::{NewConcern, Tag, Target, Text};

use crate::stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConcernState {
    concern: NewConcern,
    identified: Stamp,
}

impl ConcernState {
    pub fn correct(&mut self, text: Text) {
        self.concern = self.concern.corrected(text);
    }

    pub fn new(concern: NewConcern, identified: Stamp) -> Self {
        Self { concern, identified }
    }

    pub fn target(&self) -> Target {
        Target::Concern(self.concern.concern())
    }

    pub fn tagged(&self, tag: Tag) -> Option<&Self> {
        self.concern.tags().contains(&tag).then_some(self)
    }
}

impl fmt::Display for ConcernState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} — {}", self.concern, self.identified)
    }
}
