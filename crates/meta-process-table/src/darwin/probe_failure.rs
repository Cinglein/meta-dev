use std::path::PathBuf;
use std::str::FromStr;

use crate::scan::Scan;
use crate::table_failure::TableFailure;

#[derive(Debug, thiserror::Error)]
pub enum ProbeFailure {
    #[error("refused by the kernel")]
    Refused,
    #[error("no such process")]
    Gone,
    #[error("{0}")]
    Failed(TableFailure),
}

impl ProbeFailure {
    const REFUSED: &'static str = "errno = 1,";
    const NO_PROGRAM_FILE: &'static str = "errno = 2,";
    const GONE: &'static str = "errno = 3,";

    pub fn program(found: Result<String, Self>) -> Result<Option<PathBuf>, Self> {
        found.map(|path| Some(PathBuf::from(path))).or_else(|failure| match failure {
            Self::Failed(cause) if cause.to_string().contains(Self::NO_PROGRAM_FILE) => Ok(None),
            other => Err(other),
        })
    }

    pub fn excused(
        self,
        owned: bool,
        alive: impl FnOnce() -> Result<bool, TableFailure>,
    ) -> Result<Scan, TableFailure> {
        match self.unexplained() {
            Some(cause) if owned && alive()? => Err(cause),
            Some(_) | None => Ok(Scan::Skipped),
        }
    }

    pub fn unexplained(self) -> Option<TableFailure> {
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
            failed => Self::Failed(TableFailure::new(failed)),
        })
    }
}

impl From<String> for ProbeFailure {
    fn from(reported: String) -> Self {
        let Ok(failure) = reported.parse();
        failure
    }
}

impl From<TableFailure> for ProbeFailure {
    fn from(cause: TableFailure) -> Self {
        Self::Failed(cause)
    }
}
