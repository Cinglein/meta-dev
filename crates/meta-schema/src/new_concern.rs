use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Concern;
use crate::id::Id;
use crate::tag::Tag;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewConcern {
    concern: Id<Concern>,
    text: Text,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<Text>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,
}

impl NewConcern {
    #[must_use]
    pub fn new(text: Text, source: Option<Text>, tags: Vec<Tag>) -> Self {
        Self { concern: Id::generate(), text, source, tags }
    }

    #[must_use]
    pub fn concern(&self) -> Id<Concern> {
        self.concern
    }

    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    #[must_use]
    pub fn corrected(&self, text: Text) -> Self {
        Self { text, ..self.clone() }
    }
}

impl fmt::Display for NewConcern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.concern, self.text)?;
        self.tags.iter().try_for_each(|tag| write!(formatter, " #{tag}"))?;
        self.source.as_ref().map_or(Ok(()), |source| write!(formatter, " (from {source})"))
    }
}
