use std::fmt::{self, Write as _};

use meta_schema::Record;

use crate::skipped_line::{LineNumber, SkippedLine};
use crate::snapshot::Snapshot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Content(String);

impl Content {
    const NEWLINE: char = '\n';

    pub fn appended(&self, record: &Record) -> Result<Self, serde_json::Error> {
        serde_json::to_string(record).map(|line| Self(format!("{self}{line}{}", Self::NEWLINE)))
    }

    pub fn snapshot(&self) -> Snapshot {
        self.0.split_inclusive(Self::NEWLINE).enumerate().fold(Snapshot::default(), |snapshot, (index, line)| {
            match serde_json::from_str(line) {
                Ok(record) => snapshot.with_record(record),
                Err(error) => snapshot.with_skipped(SkippedLine::new(LineNumber::from_index(index), error)),
            }
        })
    }
}

impl From<String> for Content {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl fmt::Display for Content {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)?;
        match self.0.chars().next_back() {
            Some(last) if last != Self::NEWLINE => formatter.write_char(Self::NEWLINE),
            _ => Ok(()),
        }
    }
}

impl AsRef<str> for Content {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use meta_schema::{Event, NewTask, Record};

    use super::Content;

    const FUTURE_AND_GARBAGE: &str = "{\"at\":\"2026-10-25T00:00:00Z\",\"actor\":\"a\",\"kind\":\"Later\"}\nnot json\n";
    const SKIPPED: [&str; 2] = ["skipped meta/log line 1", "skipped meta/log line 2"];
    const UNTERMINATED: &str = "x  \n\n{}";
    const TERMINATOR: &str = "\n";
    const WRITER: &str = "writer";
    const DIVISION: &str = "features-and-maintenance";

    #[test]
    fn unknown_lines_are_listed_and_block_writes() {
        let snapshot = Content::from(FUTURE_AND_GARBAGE.to_owned()).snapshot();
        assert_eq!(snapshot.records(), []);
        assert!(snapshot.complete().is_err());
        let shown = snapshot.to_string();
        assert_eq!(shown.lines().filter_map(|line| line.split(':').next()).collect::<Vec<_>>(), SKIPPED);
    }

    #[test]
    fn appends_keep_every_earlier_byte() {
        let opened = NewTask::new(DIVISION.parse().unwrap(), WRITER.parse().unwrap(), vec![]);
        let record = Record::new(WRITER.parse().unwrap(), Event::TaskOpened(opened));
        let appended = Content::from(UNTERMINATED.to_owned()).appended(&record).unwrap();
        assert!(appended.as_ref().starts_with(&[UNTERMINATED, TERMINATOR].concat()));
        assert_eq!(appended.snapshot().records(), [record]);
        assert_eq!(appended.to_string(), appended.as_ref());
    }
}
