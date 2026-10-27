use super::ownership::Ownership;
use super::process_row::ProcessRow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgedRow {
    row: ProcessRow,
    ownership: Ownership,
}

impl JudgedRow {
    pub fn new(row: ProcessRow, ownership: Ownership) -> Self {
        Self { row, ownership }
    }

    pub fn row(&self) -> &ProcessRow {
        &self.row
    }

    pub fn owned(&self, wanted: Ownership) -> Option<&ProcessRow> {
        (self.ownership == wanted).then_some(&self.row)
    }
}
