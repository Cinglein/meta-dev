mod close_out;
mod gh_calls;
mod head_watch;
mod queue_run;

use meta_schema::PullRequest;

use crate::request::cli_failure::{CliFailure, Misuse};
use crate::request::output::Output;
use crate::request::project::Project;
use crate::request::scale_policy::ScalePolicy;
use gh_calls::GhCalls;
use head_watch::HeadWatch;
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
        let pace = project.ci_settings().queue_pace(ScalePolicy::read(project.root())?.millis().queue_floor()?)?;
        GhCalls::new(project).trunk().map_or_else(
            |stop| Ok(QueueRun::new().halted(&stop).output()),
            |trunk| {
                let watch = |pull_request: &PullRequest| HeadWatch::new(project, &trunk, pace, *pull_request);
                let ran =
                    self.pulls.iter().try_fold(QueueRun::new(), |ran, pull_request| ran.next(watch(pull_request)));
                ran.map(QueueRun::output)
            },
        )
    }
}

#[cfg(test)]
mod tests;
