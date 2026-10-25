use std::fmt;

use meta_schema::{NewIssue, Target, Text};

use crate::stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueState {
    issue: NewIssue,
    reported: Stamp,
}

impl IssueState {
    pub fn correct(&mut self, text: Text) {
        self.issue = self.issue.corrected(text);
    }

    pub fn new(issue: NewIssue, reported: Stamp) -> Self {
        Self { issue, reported }
    }

    pub fn target(&self) -> Target {
        Target::Issue(self.issue.issue())
    }
}

impl fmt::Display for IssueState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} — {}", self.issue, self.reported)
    }
}
