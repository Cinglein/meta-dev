use std::fmt;

use super::ownership::Ownership;
use super::process_row::ProcessRow;
use crate::request::local::local_settings::AgentRoot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgedRow {
    row: ProcessRow,
    ownership: Ownership,
    marked: Option<()>,
    root: Option<AgentRoot>,
}

impl JudgedRow {
    pub fn new(row: ProcessRow, ownership: Ownership, marked: Option<()>, root: Option<AgentRoot>) -> Self {
        Self { row, ownership, marked, root }
    }

    pub fn row(&self) -> &ProcessRow {
        &self.row
    }

    pub fn owned(&self, wanted: Ownership) -> Option<&ProcessRow> {
        (self.ownership == wanted).then_some(&self.row)
    }

    pub fn tracked(&self) -> Option<&ProcessRow> {
        (self.marked.is_some() && self.ownership != Ownership::Outside).then_some(&self.row)
    }

    pub fn shown(&self) -> Option<&Self> {
        (self.root.is_some() || self.ownership != Ownership::Outside).then_some(self)
    }
}

impl fmt::Display for JudgedRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = self.marked.map_or("no marker", |()| "marker");
        let root = self.root.as_ref().map_or_else(|| "no root".to_owned(), |root| format!("root {root}"));
        writeln!(formatter, "  {}, {marker}, {root}, {}", self.ownership, self.row)
    }
}
