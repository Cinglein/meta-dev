mod holders;
mod listed;

use std::fmt;

use meta_schema::Record;

pub use holders::Holders;
pub use listed::Listed;

#[derive(Debug)]
pub enum Output {
    Written(Record, Holders),
    Listed(Listed),
}

impl fmt::Display for Output {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Written(record, holders) => write!(formatter, "{record}\n{holders}"),
            Self::Listed(listed) => write!(formatter, "{listed}"),
        }
    }
}
