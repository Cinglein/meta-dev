use std::fmt;

use crate::hundredths::Hundredths;
use crate::tally::Tally;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Count(Tally),
    Median(Hundredths),
    ByLint(&'static str),
    NotImplemented,
}

impl fmt::Display for Reading {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reading::Count(count) => write!(formatter, "{count}"),
            Reading::Median(median) => write!(formatter, "median {median}"),
            Reading::ByLint(lint) => write!(formatter, "measured by lint {lint}"),
            Reading::NotImplemented => formatter.write_str("not implemented"),
        }
    }
}
