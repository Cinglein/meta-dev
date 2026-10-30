mod brief;
mod lease_line;
mod prompt;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use meta_log::LogStore;
use meta_schema::{Actor, Claim, Event};
use meta_state::{State, TaskState};

use crate::request::cli_failure::{CliFailure, Misuse};
use crate::request::output::{Listed, Output};
use crate::request::writing::Writing;
pub use brief::Brief;
pub use lease_line::LeaseLine;
pub use prompt::Prompt;

#[derive(Debug)]
pub struct Dispatch {
    claim: Claim,
    agent: Actor,
    project: Option<PathBuf>,
}

impl Dispatch {
    pub fn new(claim: Claim, agent: Actor) -> Self {
        Self { claim: claim.leased(), agent, project: None }
    }

    #[must_use]
    pub fn rooted(self, root: &Path) -> Self {
        Self { project: Some(root.to_path_buf()), ..self }
    }

    pub fn answer(self, log: &impl LogStore) -> Result<Output, CliFailure> {
        let written = Writing::new(log, self.agent.clone(), Event::TaskClaimed(self.claim.clone())).entered()?;
        let state = State::fold(log.read()?.records());
        let id = self.claim.task();
        let task = state.tasks().iter().find(|task| task.new_task().task() == id).map(TaskState::new_task);
        let lease = self.claim.lease().map(|lease| LeaseLine::new(id, lease));
        let (task, lease) =
            task.zip(lease).ok_or_else(|| CliFailure::raised::<Misuse>("claim written without a lease"))?;
        let brief = Brief::new(self.agent, task.clone(), self.project, lease);
        Ok(Output::Listed(Listed::new(written).then(brief)))
    }
}
