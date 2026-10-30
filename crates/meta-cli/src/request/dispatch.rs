mod brief;
mod lease_line;
mod prompt;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use meta_log::LogStore;
use meta_schema::{Actor, Claim, Event, Id, Lease, LeaseGrant};
use meta_state::{Refusal, State, TaskState};

use crate::request::cli_failure::{CliFailure, Invalid, Misuse};
use crate::request::output::{Listed, Output};
use crate::request::scale_policy::ScalePolicy;
use crate::request::writing::Writing;
pub use brief::Brief;
pub use lease_line::LeaseLine;
pub use prompt::Prompt;

#[derive(Debug)]
pub struct Dispatch {
    claim: Claim,
    agent: Actor,
    lease: Id<Lease>,
    project: Option<PathBuf>,
}

impl Dispatch {
    const UNISSUED: &'static str = "only an actor in [dispatch] issuers of tooling/policy.toml may run claim --agent";

    pub fn new(claim: Claim, agent: Actor) -> Self {
        Self { claim, agent, lease: Id::generate(), project: None }
    }

    #[must_use]
    pub fn rooted(self, root: &Path) -> Self {
        Self { project: Some(root.to_path_buf()), ..self }
    }

    pub fn answer(self, log: &impl LogStore, policy: &ScalePolicy, actor: Actor) -> Result<Output, CliFailure> {
        let issuer = policy.dispatch().issuer(actor).ok_or_else(|| CliFailure::raised::<Misuse>(Self::UNISSUED))?;
        let id = self.claim.task();
        let state = State::fold(log.read()?.records());
        let task = state.tasks().iter().find(|task| task.new_task().task() == id).map(TaskState::new_task).cloned();
        let task = task.ok_or_else(|| CliFailure::raised::<Invalid>(Refusal::UnknownTask(id)))?;
        let claim = self.claim.leased(LeaseGrant::new(self.lease, issuer));
        let written = Writing::new(log, self.agent.clone(), Event::TaskClaimed(claim)).entered()?;
        let brief = Brief::new(self.agent, task, self.project, LeaseLine::new(id, self.lease));
        Ok(Output::Listed(Listed::new(written).then(brief)))
    }
}
