mod holders;
mod listed;

use std::fmt;
use std::process::ExitCode;

use meta_schema::Record;

pub use holders::Holders;
pub use listed::Listed;

#[derive(Debug)]
pub enum Output {
    Written(Record, Holders),
    Listed(Listed),
    Flagged(Listed),
}

impl Output {
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Written(..) | Self::Listed(_) => ExitCode::SUCCESS,
            Self::Flagged(_) => ExitCode::FAILURE,
        }
    }
}

impl fmt::Display for Output {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Written(record, holders) => write!(formatter, "{record}\n{holders}"),
            Self::Listed(listed) | Self::Flagged(listed) => write!(formatter, "{listed}"),
        }
    }
}
