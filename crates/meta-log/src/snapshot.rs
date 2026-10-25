use std::fmt;

use meta_schema::Record;

use crate::skipped_line::SkippedLine;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    records: Vec<Record>,
    skipped: Vec<SkippedLine>,
}

impl Snapshot {
    pub fn with_record(mut self, record: Record) -> Self {
        self.records.push(record);
        self
    }

    pub fn with_skipped(mut self, line: SkippedLine) -> Self {
        self.skipped.push(line);
        self
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn skipped(&self) -> &[SkippedLine] {
        &self.skipped
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.skipped.iter().try_for_each(|line| writeln!(formatter, "skipped meta/log {line}; upgrade meta"))
    }
}
