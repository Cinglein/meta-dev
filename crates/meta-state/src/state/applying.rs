use meta_schema::{
    Block, Claim, EntryNote, EventVisitor, NewConcern, NewIssue, NewTask, SchemaVersion, TaskLink, TaskRef,
};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::state::concern_state::ConcernState;
use crate::state::issue_state::IssueState;
use crate::state::{Retired, State};
use crate::task_state::{Progress, TaskState};

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
        self.state.task_mut(claim.task())?.advance(|progress| progress.claimed(self.stamp, claim.previous_claimant()))
    }

    fn task_released(self, task: TaskRef) -> Self::Output {
        self.state.task_mut(task.task())?.advance(|progress| progress.released(&self.stamp))
    }

    fn task_blocked(self, block: Block) -> Self::Output {
        self.state.vet_blocker(block)?;
        self.state.task_mut(block.task())?.advance(|progress| progress.blocked(block.blocker(), self.stamp.actor()))
    }

    fn task_unblocked(self, task: TaskRef) -> Self::Output {
        self.state.task_mut(task.task())?.advance(|progress| progress.unblocked(self.stamp.actor()))
    }

    fn task_linked(self, link: TaskLink) -> Self::Output {
        self.state.task_mut(link.task())?.linked(link.pull_request())
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

    fn task_reopened(self, task: TaskRef) -> Self::Output {
        self.state.task_mut(task.task())?.advance(|progress| progress.reopened().map(Progress::Active))
    }
}
