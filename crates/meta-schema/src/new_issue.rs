use std::fmt;

use meta_text::Text;
use serde::{Deserialize, Serialize};

use crate::division::Division;
use crate::entity::Issue;
use crate::id::Id;
use crate::level_name::LevelName;
use crate::placement::Placement;
use crate::rating::Rating;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewIssue {
    issue: Id<Issue>,
    text: Text,
    #[serde(default, skip_serializing_if = "Placement::is_root")]
    division: Placement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    severity: Option<LevelName>,
}

impl NewIssue {
    #[must_use]
    pub fn new(division: Division, text: Text) -> Self {
        Self { issue: Id::generate(), text, division: Placement::new(division), severity: None }
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
    pub fn issue(&self) -> Id<Issue> {
        self.issue
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.division
    }

    #[must_use]
    pub fn corrected(&self, text: Text) -> Self {
        Self { text, ..self.clone() }
    }
}

impl fmt::Display for NewIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}{}", self.issue, self.text, self.division)?;
        self.severity.as_ref().map_or(Ok(()), |level| write!(formatter, " !{level}"))
    }
}
