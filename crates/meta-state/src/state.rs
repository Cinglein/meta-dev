mod anomaly;
mod applying;
mod concern_state;
mod issue_state;

use meta_schema::{Id, Record, Target, Task};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::task_state::TaskState;
use anomaly::Anomaly;
use applying::Applying;
use concern_state::ConcernState;
use issue_state::IssueState;

#[derive(Clone, Debug, Default)]
pub struct State {
    concerns: Vec<ConcernState>,
    issues: Vec<IssueState>,
    tasks: Vec<TaskState>,
    anomalies: Vec<Anomaly>,
}

impl State {
    pub fn fold(records: &[Record]) -> Self {
        records.iter().fold(Self::default(), Self::applied)
    }

    pub fn vet(&self, record: &Record) -> Result<(), Refusal> {
        record.event().visit(Applying::new(&mut self.clone(), Stamp::from_record(record)))
    }

    fn applied(mut self, record: &Record) -> Self {
        let outcome = record.event().visit(Applying::new(&mut self, Stamp::from_record(record)));
        self.anomalies.extend(outcome.err().map(|refusal| Anomaly::new(record.clone(), refusal)));
        self
    }

    pub(crate) fn add_concern(&mut self, concern: ConcernState) -> Result<(), Refusal> {
        self.fresh(concern.target())?;
        self.concerns.push(concern);
        Ok(())
    }

    pub(crate) fn add_issue(&mut self, issue: IssueState) -> Result<(), Refusal> {
        self.fresh(issue.target())?;
        self.issues.push(issue);
        Ok(())
    }

    pub(crate) fn add_task(&mut self, task: TaskState, addresses: &[Target]) -> Result<(), Refusal> {
        self.tasks
            .iter()
            .find(|known| known.new_task().task() == task.new_task().task())
            .map_or(Ok(()), |_| Err(Refusal::DuplicateId))?;
        addresses
            .iter()
            .find(|target| self.fresh(**target).is_ok())
            .map_or(Ok(()), |missing| Err(Refusal::UnknownTarget(*missing)))?;
        self.tasks.push(task);
        Ok(())
    }

    pub(crate) fn task_mut(&mut self, task: Id<Task>) -> Result<&mut TaskState, Refusal> {
        self.tasks.iter_mut().find(|known| known.new_task().task() == task).ok_or(Refusal::UnknownTask(task))
    }

    pub(crate) fn concerns(&self) -> &[ConcernState] {
        &self.concerns
    }

    pub(crate) fn issues(&self) -> &[IssueState] {
        &self.issues
    }

    pub(crate) fn tasks(&self) -> &[TaskState] {
        &self.tasks
    }

    pub(crate) fn anomalies(&self) -> &[Anomaly] {
        &self.anomalies
    }

    pub(crate) fn addressed_by(&self, target: Target) -> Option<Id<Task>> {
        self.tasks.iter().find_map(|task| task.addressing(target))
    }

    fn fresh(&self, target: Target) -> Result<(), Refusal> {
        self.concerns
            .iter()
            .map(ConcernState::target)
            .chain(self.issues.iter().map(IssueState::target))
            .find(|known| *known == target)
            .map_or(Ok(()), |_| Err(Refusal::DuplicateId))
    }
}

#[cfg(test)]
mod tests;
