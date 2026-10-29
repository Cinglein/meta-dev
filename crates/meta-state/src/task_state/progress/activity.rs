mod waiting;

use meta_schema::{Actor, Block, Blocker};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::task_state::stalling::Stalling;
use crate::task_state::task_status::TaskStatus;
use waiting::Waiting;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activity {
    assignments: Vec<Stamp>,
    waiting: Option<Block>,
    ended: Option<Blocker>,
}

impl Activity {
    pub fn unassigned() -> Self {
        Self { assignments: Vec::new(), waiting: None, ended: None }
    }

    pub fn assigned(assignments: Vec<Stamp>) -> Self {
        Self { assignments, ..Self::unassigned() }
    }

    pub fn waiting(&self) -> Option<&Block> {
        self.waiting.as_ref()
    }

    pub fn assignments(&self) -> &[Stamp] {
        &self.assignments
    }

    pub fn ended(&self) -> Option<Blocker> {
        self.ended
    }

    pub fn status(&self) -> TaskStatus {
        self.waiting.as_ref().map_or_else(
            || if self.assignments.is_empty() { TaskStatus::Open } else { TaskStatus::Assigned },
            |block| block.blocker().visit(Waiting),
        )
    }

    pub fn claimed(&self, by: Stamp, previous: Option<&Actor>) -> Result<Self, Refusal> {
        let kept = previous.map_or_else(|| Ok(self.assignments.clone()), |named| self.without(named))?;
        let actor = by.actor().clone();
        match (Self::holds(&kept, &actor), previous) {
            (false, _) => Ok(Self { assignments: [kept, vec![by]].concat(), ..self.clone() }),
            (true, None) => Ok(self.clone()),
            (true, Some(_)) => Err(Refusal::AlreadyAssigned(actor)),
        }
    }

    pub fn released(&self, by: &Stamp) -> Result<Self, Refusal> {
        let left = self.without(by.actor()).map(|kept| Self { assignments: kept, ..self.clone() })?;
        (left.assignments.is_empty() && self.waiting.is_some() && left.status().stalling() == Stalling::Possible)
            .then_some(())
            .map_or(Ok(left), |()| Err(Refusal::WaitWithoutHolder))
    }

    pub fn holder(&self, actor: &Actor) -> Result<(), Refusal> {
        self.without(actor).map(|_| ())
    }

    pub fn blocked(&self, block: &Block, by: &Actor) -> Result<Self, Refusal> {
        self.holder(by)?;
        self.waiting.as_ref().map_or_else(
            || Ok(Self { waiting: Some(block.clone()), ended: None, ..self.clone() }),
            |_| Err(Refusal::AlreadyWaiting),
        )
    }

    pub fn unblocked(&self, by: &Actor) -> Result<Self, Refusal> {
        let waiting = self.permitting(by)?.waiting.as_ref();
        waiting.map(|_| Self { waiting: None, ..self.clone() }).ok_or(Refusal::NotBlocked)
    }

    pub fn dissolved(&self) -> Self {
        Self { waiting: None, ended: self.waiting.as_ref().map(Block::blocker).or(self.ended), ..self.clone() }
    }

    fn permitting(&self, actor: &Actor) -> Result<&Self, Refusal> {
        (self.assignments.is_empty() || Self::holds(&self.assignments, actor))
            .then_some(self)
            .ok_or_else(|| Refusal::NotAssigned(actor.clone()))
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
