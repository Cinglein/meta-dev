use std::fmt;

use crate::request::ci::Commit;
use crate::request::output::Listed;

#[derive(Debug)]
pub enum StopReason {
    Unknown(Listed),
    Unwritable(String),
    Unpushed(Commit),
    Unwatched(Commit),
    Known(Listed),
}

impl fmt::Display for StopReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(reason) | Self::Known(reason) => write!(formatter, "{reason}"),
            Self::Unwritable(error) => write!(formatter, "CI state unwritable: {error}; fix it or ask the owner"),
            Self::Unpushed(commit) => {
                write!(formatter, "HEAD {commit} is not pushed: push it, then run `meta ci-wait`; or ask the owner")
            }
            Self::Unwatched(commit) => write!(formatter, "CI not watched for {commit}: push again to watch it"),
        }
    }
}
