use std::fmt;

use meta_schema::Record;

use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anomaly {
    record: Record,
    refusal: Refusal,
}

impl Anomaly {
    pub fn new(record: Record, refusal: Refusal) -> Self {
        Self { record, refusal }
    }
}

impl fmt::Display for Anomaly {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.record, self.refusal)
    }
}
