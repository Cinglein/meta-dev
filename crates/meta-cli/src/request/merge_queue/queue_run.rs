use meta_pulls::{Detour, Stop};
use meta_schema::PullRequest;

use super::head_watch::HeadWatch;
use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};
use crate::request::project::Project;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueRun {
    shown: Listed,
    detours: Option<Listed>,
    halted: Option<()>,
}

impl QueueRun {
    pub fn new() -> Self {
        Self { shown: Listed::new(""), detours: None, halted: None }
    }

    pub fn next(self, project: &Project, pull_request: PullRequest) -> Result<Self, CliFailure> {
        match self.halted {
            Some(()) => Ok(self),
            None => HeadWatch::new(project, pull_request).watched().map(|watched| watched.folded(self)),
        }
    }

    pub fn landed(self, shown: Listed) -> Self {
        Self { shown: self.shown.then(shown), ..self }
    }

    pub fn detoured(self, detour: Detour) -> Self {
        let pull_request = detour.pull_request();
        let detours = self.detours.map_or_else(
            || Listed::new(format_args!("skipped, needs a fix: {pull_request}")),
            |detours| detours.then(format_args!(", {pull_request}")),
        );
        Self { shown: self.shown.then(format_args!("{detour}\n")), detours: Some(detours), ..self }
    }

    pub fn halted(self, stop: &Stop) -> Self {
        Self { shown: self.shown.then(format_args!("{stop}\n")), halted: Some(()), ..self }
    }

    pub fn output(self) -> Output {
        match (self.halted, self.detours) {
            (None, None) => Output::Listed(self.shown),
            (Some(()), None) => Output::Flagged(self.shown),
            (_, Some(detours)) => Output::Flagged(self.shown.then(detours).then("\n")),
        }
    }
}
