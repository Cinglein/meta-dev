use std::fmt;

use chrono::{DateTime, Utc};
use meta_schema::{Actor, Record};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    by: Actor,
    at: DateTime<Utc>,
}

impl Stamp {
    pub fn from_record(record: &Record) -> Self {
        Self { by: record.actor().clone(), at: record.written_at() }
    }

    pub fn actor(&self) -> &Actor {
        &self.by
    }
}

impl fmt::Display for Stamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.by, self.at.format("%Y-%m-%d %H:%MZ"))
    }
}
