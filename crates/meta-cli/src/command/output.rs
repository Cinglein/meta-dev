mod listed;

use std::fmt;

use meta_schema::Record;

pub use listed::Listed;

#[derive(Debug)]
pub enum Output {
    Written(Record),
    Listed(Listed),
}

impl fmt::Display for Output {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Written(record) => writeln!(formatter, "{record}"),
            Self::Listed(listed) => write!(formatter, "{listed}"),
        }
    }
}
