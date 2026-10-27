use std::fmt;

use serde::{Deserialize, Serialize};

use crate::division::Division;
use crate::entity::Concern;
use crate::id::Id;
use crate::level_name::LevelName;
use crate::placement::Placement;
use crate::rating::Rating;
use crate::tag::Tag;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewConcern {
    concern: Id<Concern>,
    text: Text,
    #[serde(default, skip_serializing_if = "Placement::is_root")]
    division: Placement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<Text>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    severity: Option<LevelName>,
}

impl NewConcern {
    #[must_use]
    pub fn new(division: Division, text: Text, source: Option<Text>, tags: Vec<Tag>) -> Self {
        Self { concern: Id::generate(), text, division: Placement::new(division), source, tags, severity: None }
    }

    #[must_use]
    pub fn rated(self, level: LevelName) -> Self {
        Self { severity: Some(level), ..self }
    }

    #[must_use]
    pub fn placed(self, division: Division) -> Self {
        Self { division: Placement::new(division), ..self }
    }

    #[must_use]
    pub fn rating(&self) -> Rating {
        Rating::from(self.severity.clone())
    }

    #[must_use]
    pub fn concern(&self) -> Id<Concern> {
        self.concern
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.division
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
        write!(formatter, "{} {}{}", self.concern, self.text, self.division)?;
        self.severity.as_ref().map_or(Ok(()), |level| write!(formatter, " !{level}"))?;
        self.tags.iter().try_for_each(|tag| write!(formatter, " #{tag}"))?;
        self.source.as_ref().map_or(Ok(()), |source| write!(formatter, " (from {source})"))
    }
}
