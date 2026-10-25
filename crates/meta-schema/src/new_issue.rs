use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Issue;
use crate::id::Id;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewIssue {
    issue: Id<Issue>,
    text: Text,
}

impl NewIssue {
    #[must_use]
    pub fn new(text: Text) -> Self {
        Self { issue: Id::generate(), text }
    }

    #[must_use]
    pub fn issue(&self) -> Id<Issue> {
        self.issue
    }

    #[must_use]
    pub fn corrected(&self, text: Text) -> Self {
        Self { text, ..self.clone() }
    }
}

impl fmt::Display for NewIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.issue, self.text)
    }
}
