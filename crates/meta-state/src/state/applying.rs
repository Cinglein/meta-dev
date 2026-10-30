use meta_schema::{
    Acceptance, Block, Claim, EntryNote, EventVisitor, NewConcern, NewIssue, NewProject, NewTask, Observation,
    Relocation, Rerating, SchemaVersion, TaskLink, TaskPurpose, TaskRef,
};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::state::concern_state::ConcernState;
use crate::state::issue_state::IssueState;
use crate::state::{Retired, State};
use crate::task_state::{Progress, Reopening, TaskState};

#[derive(Debug)]
pub struct Applying<'state> {
    state: &'state mut State,
    stamp: Stamp,
}

impl<'state> Applying<'state> {
    pub fn new(state: &'state mut State, stamp: Stamp) -> Self {
        Self { state, stamp }
    }
}

impl EventVisitor for Applying<'_> {
    type Output = Result<(), Refusal>;

    fn concern_identified(self, concern: &NewConcern) -> Self::Output {
        self.state.add_concern(ConcernState::new(concern.clone(), self.stamp))
    }

    fn issue_reported(self, issue: &NewIssue) -> Self::Output {
        self.state.add_issue(IssueState::new(issue.clone(), self.stamp))
    }

    fn task_opened(self, task: &NewTask) -> Self::Output {
        self.state.add_task(TaskState::new(task.clone(), self.stamp), task.addresses())
    }

    fn task_claimed(self, claim: &Claim) -> Self::Output {
        let stamp = self.stamp.leased(claim.grant().cloned());
        self.state.task_mut(claim.task())?.advance(|progress| progress.claimed(stamp, claim.previous_claimant()))
    }

    fn task_released(self, task: TaskRef) -> Self::Output {
        self.state.task_mut(task.task())?.advance(|progress| progress.released(&self.stamp))
    }

    fn task_blocked(self, block: &Block) -> Self::Output {
        self.state.vet_blocker(block)?;
        self.state.task_mut(block.task())?.advance(|progress| progress.blocked(block, self.stamp.actor()))
    }

    fn task_unblocked(self, task: TaskRef) -> Self::Output {
        self.state.task_mut(task.task())?.advance(|progress| progress.unblocked(self.stamp.actor()))
    }

    fn task_linked(self, link: TaskLink) -> Self::Output {
        self.state.task_mut(link.task()).map(|task| task.linked(link.pull_request()))
    }

    fn task_done(self, task: TaskRef) -> Self::Output {
        self.state.task_mut(task.task())?.advance(|progress| progress.finished(self.stamp))
    }

    fn entry_retired(self, note: &EntryNote) -> Self::Output {
        self.state.retire(Retired::new(note.clone(), self.stamp))
    }

    fn entry_corrected(self, note: &EntryNote) -> Self::Output {
        self.state.correct(note)
    }

    fn schema_versioned(self, version: SchemaVersion) -> Self::Output {
        self.state.note_schema(version);
        Ok(())
    }

    fn item_rated(self, rerating: &Rerating) -> Self::Output {
        self.state.rerate(rerating)
    }

    fn item_moved(self, relocation: &Relocation) -> Self::Output {
        self.state.relocate(relocation)
    }

    fn task_reopened(self, task: TaskRef) -> Self::Output {
        let reopening = Reopening::under(self.state.schema());
        self.state.task_mut(task.task())?.advance(|progress| progress.reopened(reopening).map(Progress::Active))
    }

    fn project_proposed(self, _: &NewProject) -> Self::Output {
        Ok(())
    }

    fn project_accepted(self, _: &Acceptance) -> Self::Output {
        Ok(())
    }

    fn project_observed(self, _: &Observation) -> Self::Output {
        Ok(())
    }

    fn task_purposed(self, purpose: TaskPurpose) -> Self::Output {
        self.state.task_mut(purpose.task()).map(|_| ())
    }
}
