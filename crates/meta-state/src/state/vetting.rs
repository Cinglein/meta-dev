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
}

impl BlockerVisitor for Vetting<'_> {
    type Output = Result<(), Refusal>;

    fn discussion(self, issue: Id<Issue>) -> Self::Output {
        self.state.known(Entry::Issue(issue)).map_err(|_| Refusal::UnknownTarget(Target::Issue(issue)))
    }

    fn task(self, task: Id<Task>) -> Self::Output {
        if task == self.own {
            Err(Refusal::WaitsOnItself)
        } else {
            self.state.known(Entry::Task(task)).map_err(|_| Refusal::UnknownTask(task))
        }
    }

    fn external(self, _kind: ExternalKind) -> Self::Output {
        Ok(())
    }
}
