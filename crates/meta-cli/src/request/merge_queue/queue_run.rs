use meta_pulls::Stop;
use meta_schema::PullRequest;

use super::head_watch::HeadWatch;
use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueRun {
    shown: Listed,
    flagged: Option<Listed>,
    halted: Option<()>,
}

impl QueueRun {
    pub fn new() -> Self {
        Self { shown: Listed::new(""), flagged: None, halted: None }
    }

    pub fn next(self, watch: HeadWatch<'_>) -> Result<Self, CliFailure> {
        match self.halted {
            Some(()) => Ok(self),
            None => watch.watched().map(|watched| watched.folded(self)),
        }
    }

    pub fn landed(self, shown: Listed) -> Self {
        Self { shown: self.shown.then(shown), ..self }
    }

    pub fn flagged(self, shown: Listed, pull_request: PullRequest) -> Self {
        let flagged = self.flagged.map_or_else(
            || Listed::new(format_args!("needs attention: {pull_request}")),
            |flagged| flagged.then(format_args!(", {pull_request}")),
        );
        Self { shown: self.shown.then(shown), flagged: Some(flagged), ..self }
    }

    pub fn halted(self, stop: &Stop) -> Self {
        Self { shown: self.shown.then(format_args!("{stop}\n")), halted: Some(()), ..self }
    }

    pub fn output(self) -> Output {
        match (self.halted, self.flagged) {
            (None, None) => Output::Listed(self.shown),
            (Some(()), None) => Output::Flagged(self.shown),
            (_, Some(flagged)) => Output::Flagged(self.shown.then(flagged).then("\n")),
        }
    }
}
