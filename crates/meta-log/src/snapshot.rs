use std::fmt;

use meta_schema::Record;

use crate::log_failure::LogFailure;
use crate::skipped_line::SkippedLine;

#[derive(Debug, Default)]
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

    pub fn complete(&self) -> Result<&[Record], LogFailure> {
        self.skipped.first().map_or(Ok(&self.records), |unread| Err(LogFailure::Unreadable(unread.line())))
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.skipped.iter().try_for_each(|line| writeln!(formatter, "skipped meta/log {line}; upgrade meta"))
    }
}
