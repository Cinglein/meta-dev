use meta_schema::{Claim, EventVisitor, NewConcern, NewIssue, NewTask, TaskRef};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::state::State;
use crate::state::concern_state::ConcernState;
use crate::state::issue_state::IssueState;
use crate::task_state::TaskState;

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
        let task = self.state.task_mut(claim.task())?;
        task.advance(task.status().claimed(self.stamp, claim.previous_claimant()))
    }

    fn task_released(self, task: TaskRef) -> Self::Output {
        let task_state = self.state.task_mut(task.task())?;
        task_state.advance(task_state.status().released(&self.stamp))
    }

    fn task_done(self, task: TaskRef) -> Self::Output {
        let task_state = self.state.task_mut(task.task())?;
        task_state.advance(task_state.status().finished(self.stamp))
    }

    fn task_reopened(self, task: TaskRef) -> Self::Output {
        let task_state = self.state.task_mut(task.task())?;
        task_state.advance(task_state.status().reopened())
    }
}
