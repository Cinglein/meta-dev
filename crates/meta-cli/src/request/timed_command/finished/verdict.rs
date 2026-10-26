use super::Finished;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::{Listed, Output};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Flagged,
    Failed,
}

impl Verdict {
    pub fn answered(self, finished: &Finished) -> Result<Output, CliFailure> {
        match self {
            Self::Clean => Ok(Output::Listed(Listed::new(finished.output()))),
            Self::Flagged => Ok(Output::Flagged(Listed::new(finished.output()))),
            Self::Failed => Err(CliFailure::raised::<Setup>(finished.complaint())),
        }
    }
}
