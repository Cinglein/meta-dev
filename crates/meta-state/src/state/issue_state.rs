use std::fmt;

use meta_schema::{Division, LevelName, NewIssue, Target, Text};

use crate::rated_item::RatedItem;
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

    pub fn rerate(&mut self, level: LevelName) {
        self.issue = self.issue.clone().rated(level);
    }

    pub fn relocate(&mut self, division: Division, level: Option<LevelName>) {
        self.issue = level.into_iter().fold(self.issue.clone().placed(division), NewIssue::rated);
    }

    pub fn new(issue: NewIssue, reported: Stamp) -> Self {
        Self { issue, reported }
    }

    pub fn target(&self) -> Target {
        Target::Issue(self.issue.issue())
    }

    pub fn rated(&self) -> RatedItem {
        RatedItem::new(self.target(), self.issue.placement().clone(), self.issue.rating())
    }

    pub fn new_issue(&self) -> &NewIssue {
        &self.issue
    }
}

impl fmt::Display for IssueState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} — {}", self.issue, self.reported)
    }
}
