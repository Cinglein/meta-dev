use std::str::FromStr;

use crate::request::local::local_clone::vetting::cause::Cause;

#[derive(Debug, thiserror::Error)]
pub enum ProbeFailure {
    #[error("refused by the kernel")]
    Refused,
    #[error("no such process")]
    Gone,
    #[error("{0}")]
    Failed(Cause),
}

impl ProbeFailure {
    const REFUSED: &'static str = "errno = 1,";
    const GONE: &'static str = "errno = 3,";

    pub fn unexplained(self) -> Option<Cause> {
        match self {
            Self::Refused | Self::Gone => None,
            Self::Failed(cause) => Some(cause),
        }
    }
}

impl FromStr for ProbeFailure {
    type Err = std::convert::Infallible;

    fn from_str(reported: &str) -> Result<Self, Self::Err> {
        Ok(match reported {
            refused if refused.contains(Self::REFUSED) => Self::Refused,
            gone if gone.contains(Self::GONE) => Self::Gone,
            failed => Self::Failed(Cause::new(failed)),
        })
    }
}

impl From<String> for ProbeFailure {
    fn from(reported: String) -> Self {
        let Ok(failure) = reported.parse();
        failure
    }
}

impl From<Cause> for ProbeFailure {
    fn from(cause: Cause) -> Self {
        Self::Failed(cause)
    }
}
