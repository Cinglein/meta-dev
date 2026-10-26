use std::fmt;

use serde::{Deserialize, Serialize};

use crate::division::Division;
use crate::entity::Issue;
use crate::id::Id;
use crate::placement::Placement;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewIssue {
    issue: Id<Issue>,
    text: Text,
    #[serde(default, skip_serializing_if = "Placement::is_root")]
    division: Placement,
}

impl NewIssue {
    #[must_use]
    pub fn new(division: Division, text: Text) -> Self {
        Self { issue: Id::generate(), text, division: Placement::new(division) }
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
        write!(formatter, "{} {}{}", self.issue, self.text, self.division)
    }
}
