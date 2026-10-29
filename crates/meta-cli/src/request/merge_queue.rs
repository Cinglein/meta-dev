mod close_out;
mod head_watch;
mod queue_run;
mod watched;

use meta_schema::PullRequest;

use crate::request::cli_failure::{CliFailure, Misuse};
use crate::request::output::Output;
use crate::request::project::Project;
use queue_run::QueueRun;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergeQueue {
    pulls: Vec<PullRequest>,
}

impl MergeQueue {
    pub fn new(pulls: Vec<PullRequest>) -> Result<Self, CliFailure> {
        match pulls.as_slice() {
            [] => Err(CliFailure::raised::<Misuse>("expected pull request numbers, head first")),
            [..] => Ok(Self { pulls }),
        }
    }

    pub fn run(self, project: &Project) -> Result<Output, CliFailure> {
        let ran = self.pulls.iter().try_fold(QueueRun::new(), |ran, pull_request| ran.next(project, *pull_request));
        ran.map(QueueRun::output)
    }
}

#[cfg(test)]
mod tests;
