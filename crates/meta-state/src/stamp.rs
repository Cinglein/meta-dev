use std::fmt;

use chrono::{DateTime, Utc};
use meta_schema::{Actor, Id, Lease, Record};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    by: Actor,
    at: DateTime<Utc>,
    lease: Option<Id<Lease>>,
}

impl Stamp {
    pub fn from_record(record: &Record) -> Self {
        Self { by: record.actor().clone(), at: record.written_at(), lease: None }
    }

    #[must_use]
    pub fn leased(self, lease: Option<Id<Lease>>) -> Self {
        Self { lease, ..self }
    }

    #[must_use]
    pub fn renewed(self, by: &Self) -> Self {
        match (&self.by == by.actor(), by.lease) {
            (true, Some(lease)) => Self { lease: Some(lease), ..self },
            _ => self,
        }
    }

    pub fn actor(&self) -> &Actor {
        &self.by
    }

    pub fn lease(&self) -> Option<Id<Lease>> {
        self.lease
    }
}

impl fmt::Display for Stamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.by, self.at.format("%Y-%m-%d %H:%MZ"))?;
        self.lease.map_or(Ok(()), |_| formatter.write_str(" leased"))
    }
}
