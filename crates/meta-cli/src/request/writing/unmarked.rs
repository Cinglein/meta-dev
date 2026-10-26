use meta_log::LogFailure;
use meta_state::Refusal;

use crate::request::cli_failure::{CliFailure, Setup};

#[derive(Debug)]
pub enum Unmarked {
    Missing,
    Failed(CliFailure),
}

impl Unmarked {
    pub fn recovered<Done>(self, mark: impl FnOnce() -> Result<Done, CliFailure>) -> Result<Done, CliFailure> {
        match self {
            Self::Missing => mark(),
            Self::Failed(failure) => Err(failure),
        }
    }
}

impl From<LogFailure> for Unmarked {
    fn from(failure: LogFailure) -> Self {
        Self::Failed(failure.into())
    }
}

impl From<Refusal> for Unmarked {
    fn from(refusal: Refusal) -> Self {
        Self::Failed(refusal.into())
    }
}

impl From<Unmarked> for CliFailure {
    fn from(unmarked: Unmarked) -> Self {
        unmarked
            .recovered(|| Err(CliFailure::raised::<Setup>("the schema marker was not kept")))
            .unwrap_or_else(|failure| failure)
    }
}
