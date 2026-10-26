use std::iter::successors;

use meta_schema::{BlockerVisitor, Entry, ExternalKind, Id, Issue, Target, Task};

use crate::refusal::Refusal;
use crate::state::State;

#[derive(Debug)]
pub struct Vetting<'state> {
    state: &'state State,
    own: Id<Task>,
}

impl<'state> Vetting<'state> {
    pub fn new(state: &'state State, own: Id<Task>) -> Self {
        Self { state, own }
    }

    fn open(&self, entry: Entry) -> Result<(), Refusal> {
        self.state.closed(entry).map_or(Ok(()), |()| Err(Refusal::Closed(entry)))
    }
}

impl BlockerVisitor for Vetting<'_> {
    type Output = Result<(), Refusal>;

    fn discussion(self, issue: Id<Issue>) -> Self::Output {
        let entry = Entry::Issue(issue);
        self.state
            .known(entry)
            .map_err(|_| Refusal::UnknownTarget(Target::Issue(issue)))
            .and_then(|()| self.open(entry))
    }

    fn task(self, task: Id<Task>) -> Self::Output {
        let entry = Entry::Task(task);
        match (task == self.own, successors(Some(task), |waited| self.state.waits_on(*waited)).any(|w| w == self.own)) {
            (true, _) => Err(Refusal::WaitsOnItself),
            (false, true) => Err(Refusal::WaitCycle),
            (false, false) => {
                self.state.known(entry).map_err(|_| Refusal::UnknownTask(task)).and_then(|()| self.open(entry))
            }
        }
    }

    fn external(self, _kind: ExternalKind) -> Self::Output {
        Ok(())
    }
}
