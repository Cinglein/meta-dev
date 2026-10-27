use std::fmt;

use meta_schema::{Division, LevelName, NewConcern, Tag, Target, Text};

use crate::rated_item::RatedItem;
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

    pub fn rerate(&mut self, level: LevelName) {
        self.concern = self.concern.clone().rated(level);
    }

    pub fn relocate(&mut self, division: Division, level: Option<LevelName>) {
        self.concern = level.into_iter().fold(self.concern.clone().placed(division), NewConcern::rated);
    }

    pub fn new(concern: NewConcern, identified: Stamp) -> Self {
        Self { concern, identified }
    }

    pub fn target(&self) -> Target {
        Target::Concern(self.concern.concern())
    }

    pub fn rated(&self) -> RatedItem {
        RatedItem::new(self.target(), self.concern.placement().clone(), self.concern.rating())
    }

    pub fn new_concern(&self) -> &NewConcern {
        &self.concern
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
