mod waiting;

use meta_schema::{Actor, Blocker};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::task_state::task_status::TaskStatus;
use waiting::Waiting;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Activity {
    assignments: Vec<Stamp>,
    blocker: Option<Blocker>,
}

impl Activity {
    pub fn assignments(&self) -> &[Stamp] {
        &self.assignments
    }

    pub fn status(&self) -> TaskStatus {
        self.blocker.map_or_else(
            || if self.assignments.is_empty() { TaskStatus::Open } else { TaskStatus::Assigned },
            |blocker| blocker.visit(Waiting),
        )
    }

    pub fn claimed(&self, by: Stamp, previous: Option<&Actor>) -> Result<Self, Refusal> {
        let kept = previous.map_or_else(|| Ok(self.assignments.clone()), |named| self.without(named))?;
        let actor = by.actor().clone();
        Self::holds(&kept, &actor).then_some(()).map_or_else(
            || Ok(Self { assignments: [kept, vec![by]].concat(), blocker: self.blocker }),
            |()| Err(Refusal::AlreadyAssigned(actor)),
        )
    }

    pub fn released(&self, by: &Actor) -> Result<Self, Refusal> {
        self.without(by).map(|kept| Self { assignments: kept, blocker: self.blocker })
    }

    pub fn blocked(&self, on: Blocker) -> Self {
        Self { assignments: self.assignments.clone(), blocker: Some(on) }
    }

    pub fn unblocked(&self) -> Result<Self, Refusal> {
        self.blocker.map(|_| Self { assignments: self.assignments.clone(), blocker: None }).ok_or(Refusal::NotBlocked)
    }

    fn holds(assignments: &[Stamp], actor: &Actor) -> bool {
        assignments.iter().any(|held| held.actor() == actor)
    }

    fn without(&self, actor: &Actor) -> Result<Vec<Stamp>, Refusal> {
        Self::holds(&self.assignments, actor)
            .then(|| self.assignments.iter().filter(|held| held.actor() != actor).cloned().collect())
            .ok_or_else(|| Refusal::NotAssigned(actor.clone()))
    }
}
