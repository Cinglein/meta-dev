use std::fmt;
use std::str::FromStr;

use meta_schema::Record;

use crate::detail::Detail;
use crate::skipped_line::{LineNumber, SkippedLine};
use crate::snapshot::Snapshot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Content(String);

impl Content {
    pub fn appended(&self, record: &Record) -> Result<Self, serde_json::Error> {
        serde_json::to_string(record).map(|line| Self(format!("{}{line}\n", self.0)))
    }

    pub fn terminated(self) -> Self {
        match self.0.as_str() {
            "" => self,
            text => Self(format!("{text}\n")),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.0.lines().enumerate().fold(Snapshot::default(), |snapshot, (index, line)| {
            match serde_json::from_str(line) {
                Ok(record) => snapshot.with_record(record),
                Err(error) => {
                    snapshot.with_skipped(SkippedLine::new(LineNumber::from_index(index), Detail::from_reason(&error)))
                }
            }
        })
    }
}

impl From<String> for Content {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl FromStr for Content {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.to_owned()))
    }
}

impl fmt::Display for Content {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Content;

    const FUTURE_AND_GARBAGE: &str = "{\"at\":\"2026-10-25T00:00:00Z\",\"actor\":\"a\",\"kind\":\"Later\"}\nnot json\n";
    const SKIPPED: [&str; 2] = ["skipped meta/log line 1", "skipped meta/log line 2"];

    #[test]
    fn unknown_lines_are_listed_and_block_writes() {
        let snapshot = FUTURE_AND_GARBAGE.parse::<Content>().unwrap().snapshot();
        assert_eq!(snapshot.records(), []);
        assert!(snapshot.complete().is_err());
        let shown = snapshot.to_string();
        assert_eq!(shown.lines().filter_map(|line| line.split(':').next()).collect::<Vec<_>>(), SKIPPED);
    }
}
