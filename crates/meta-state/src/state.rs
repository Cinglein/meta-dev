mod anomaly;
mod applying;
mod concern_state;
mod issue_state;
mod retired;
mod vetting;

use std::collections::HashMap;

use meta_schema::{
    Actor, Block, Entry, EntryNote, Id, PullRequest, Record, Relocation, Rerating, SchemaVersion, Target, Task,
    TaskLink, Text,
};

use crate::live_lease::LiveLease;
use crate::rated_item::RatedItem;
use crate::refusal::Refusal;
use crate::stamp::Stamp;
use crate::task_state::{TaskState, TaskStatus};
use anomaly::Anomaly;
use applying::Applying;
use concern_state::ConcernState;
use issue_state::IssueState;
pub use retired::Retired;
use vetting::Vetting;

#[derive(Clone, Debug, Default)]
pub struct State {
    concerns: Vec<ConcernState>,
    issues: Vec<IssueState>,
    tasks: Vec<TaskState>,
    index_of: HashMap<Entry, usize>,
    addressers: HashMap<Target, Vec<Id<Task>>>,
    anomalies: Vec<Anomaly>,
    retired: HashMap<Entry, Retired>,
    schema: Option<SchemaVersion>,
}

impl State {
    pub fn fold(records: &[Record]) -> Self {
        records.iter().fold(Self::default(), Self::applied)
    }

    pub fn vet(&self, record: &Record) -> Result<(), Refusal> {
        self.tried(record).map(|_| ())
    }

    pub fn tried(&self, record: &Record) -> Result<Self, Refusal> {
        self.schema
            .filter(|marked| *marked > SchemaVersion::current())
            .map_or(Ok(()), |marked| Err(Refusal::NewerSchema(marked)))?;
        let linked = |link: &TaskLink| {
            self.task(link.task()).is_some_and(|task| task.pull_requests().contains(&link.pull_request()))
        };
        let known = record.event().task_link().filter(linked);
        known.map_or(Ok(()), |link| Err(Refusal::AlreadyLinked(link.pull_request())))?;
        let mut next = self.clone();
        record.event().visit(Applying::new(&mut next, Stamp::from_record(record))).map(|()| next.swept())
    }

    #[must_use]
    pub fn schema(&self) -> Option<SchemaVersion> {
        self.schema
    }

    pub(crate) fn note_schema(&mut self, version: SchemaVersion) {
        self.schema = self.schema.max(Some(version));
    }

    pub(crate) fn swept(mut self) -> Self {
        let ended: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                task.progress().status().waiting_on_entry().is_some_and(|entry| self.closed(entry).is_some())
            })
            .map(|(index, _)| index)
            .collect();
        self.tasks
            .iter_mut()
            .enumerate()
            .filter(|(index, _)| ended.contains(index))
            .for_each(|(_, task)| task.dissolve());
        self
    }

    #[must_use]
    pub fn holders(&self, task: Id<Task>) -> Vec<Actor> {
        self.task(task)
            .map(|task| task.progress().assignments().iter().map(|stamp| stamp.actor().clone()).collect())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn leases(&self, task: Id<Task>) -> Vec<LiveLease> {
        let live = self.retirement(Entry::Task(task)).map_or_else(|| self.task(task), |_| None);
        let held = live.map(|task| task.progress().assignments()).unwrap_or_default();
        held.iter()
            .filter_map(|stamp| stamp.lease().map(|lease| LiveLease::new(stamp.actor().clone(), lease.clone())))
            .collect()
    }

    #[must_use]
    pub fn unlinked(&self, task: Id<Task>, pull_requests: &[PullRequest]) -> Vec<TaskLink> {
        self.task(task)
            .filter(|held| !held.progress().assignments().is_empty())
            .map(|held| {
                let fresh = pull_requests.iter().filter(|pull_request| !held.pull_requests().contains(pull_request));
                fresh.map(|pull_request| TaskLink::new(task, *pull_request)).collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn waits_on(&self, task: Id<Task>) -> Option<Id<Task>> {
        self.task(task).and_then(|task| self.status_of(task).waiting_on_task())
    }

    fn applied(mut self, record: &Record) -> Self {
        let outcome = record.event().visit(Applying::new(&mut self, Stamp::from_record(record)));
        self.anomalies.extend(outcome.err().map(|refusal| Anomaly::new(record.clone(), refusal)));
        self.swept()
    }

    pub(crate) fn add_concern(&mut self, concern: ConcernState) -> Result<(), Refusal> {
        self.fresh(concern.target())?;
        self.index_of.insert(concern.target().entry(), self.concerns.len());
        self.concerns.push(concern);
        Ok(())
    }

    pub(crate) fn add_issue(&mut self, issue: IssueState) -> Result<(), Refusal> {
        self.fresh(issue.target())?;
        self.index_of.insert(issue.target().entry(), self.issues.len());
        self.issues.push(issue);
        Ok(())
    }

    pub(crate) fn add_task(&mut self, task: TaskState, addresses: &[Target]) -> Result<(), Refusal> {
        self.known(task.entry()).map_or(Ok(()), |()| Err(Refusal::DuplicateId))?;
        addresses
            .iter()
            .find(|target| self.fresh(**target).is_ok())
            .map_or(Ok(()), |missing| Err(Refusal::UnknownTarget(*missing)))?;
        let id = task.new_task().task();
        self.addressers = addresses.iter().fold(std::mem::take(&mut self.addressers), |mut addressers, target| {
            addressers.entry(*target).or_default().push(id);
            addressers
        });
        self.index_of.insert(task.entry(), self.tasks.len());
        self.tasks.push(task);
        Ok(())
    }

    pub(crate) fn task_mut(&mut self, task: Id<Task>) -> Result<&mut TaskState, Refusal> {
        self.retirement(Entry::Task(task)).map_or(Ok(()), |retired| Err(Refusal::Retired(retired.note().entry())))?;
        self.index_of
            .get(&Entry::Task(task))
            .and_then(|index| self.tasks.get_mut(*index))
            .ok_or(Refusal::UnknownTask(task))
    }

    pub(crate) fn vet_blocker(&self, block: &Block) -> Result<(), Refusal> {
        block.blocker().visit(Vetting::new(self, block.task()))
    }

    #[must_use]
    pub fn status_of(&self, task: &TaskState) -> TaskStatus {
        self.retirement(task.entry()).map_or_else(|| task.progress().status(), |_| TaskStatus::Retired)
    }

    pub(crate) fn retire(&mut self, retired: Retired) -> Result<(), Refusal> {
        let entry = retired.note().entry();
        self.known(entry)?;
        self.retirement(entry).map_or(Ok(()), |done| Err(Refusal::Retired(done.note().entry())))?;
        self.retired.insert(entry, retired);
        Ok(())
    }

    pub(crate) fn correct(&mut self, note: &EntryNote) -> Result<(), Refusal> {
        let entry = note.entry();
        let index = self.index_of.get(&entry).copied().ok_or(Refusal::UnknownEntry(entry))?;
        let text: &Text = note.text();
        self.concerns
            .get_mut(index)
            .filter(|known| known.target().entry() == entry)
            .map(|known| known.correct(text.clone()))
            .or_else(|| {
                self.issues
                    .get_mut(index)
                    .filter(|known| known.target().entry() == entry)
                    .map(|known| known.correct(text.clone()))
            })
            .or_else(|| {
                self.tasks
                    .get_mut(index)
                    .filter(|known| known.entry() == entry)
                    .map(|known| known.correct(text.clone()))
            })
            .ok_or(Refusal::UnknownEntry(entry))
    }

    pub(crate) fn rerate(&mut self, rerating: &Rerating) -> Result<(), Refusal> {
        let item = rerating.target();
        let change = rerating.change();
        let rated = self.rated_item(item).ok_or(Refusal::UnknownTarget(item))?;
        self.retirement(item.entry()).map_or(Ok(()), |_| Err(Refusal::Retired(item.entry())))?;
        let moved = rerating.seen_in().filter(|seen| rated.placement().division() != Some(*seen));
        moved.map_or(Ok(()), |_| Err(Refusal::MovedSince))?;
        let current = rated.rating().clone();
        (current == change.before()).then_some(()).ok_or(Refusal::RatedSince(current))?;
        let index = self.index_of.get(&item.entry()).copied().unwrap_or_default();
        let level = change.level();
        self.concerns
            .get_mut(index)
            .filter(|known| known.target() == item)
            .map(|known| known.rerate(level.clone()))
            .or_else(|| {
                let issue = self.issues.get_mut(index).filter(|known| known.target() == item);
                issue.map(|known| known.rerate(level.clone()))
            })
            .ok_or(Refusal::UnknownTarget(item))
    }

    pub(crate) fn relocate(&mut self, relocation: &Relocation) -> Result<(), Refusal> {
        let item = relocation.target();
        let current = self.rated_item(item).ok_or(Refusal::UnknownTarget(item))?;
        self.retirement(item.entry()).map_or(Ok(()), |_| Err(Refusal::Retired(item.entry())))?;
        let division = relocation.division();
        (current.placement() == relocation.origin()).then_some(()).ok_or(Refusal::MovedSince)?;
        (current.placement().division() != Some(division))
            .then_some(())
            .ok_or_else(|| Refusal::AlreadyIn(division.clone()))?;
        let carry = relocation.carry();
        (*current.rating() == carry.before())
            .then_some(())
            .ok_or_else(|| Refusal::RatedSince(current.rating().clone()))?;
        let index = self.index_of.get(&item.entry()).copied().unwrap_or_default();
        let level = carry.level();
        self.concerns
            .get_mut(index)
            .filter(|known| known.target() == item)
            .map(|known| known.relocate(division.clone(), level.cloned()))
            .or_else(|| {
                let issue = self.issues.get_mut(index).filter(|known| known.target() == item);
                issue.map(|known| known.relocate(division.clone(), level.cloned()))
            })
            .ok_or(Refusal::UnknownTarget(item))
    }

    #[must_use]
    pub fn rated_item(&self, item: Target) -> Option<RatedItem> {
        let index = self.index_of.get(&item.entry()).copied()?;
        let concern = self.concerns.get(index).filter(|known| known.target() == item).map(ConcernState::rated);
        concern.or_else(|| self.issues.get(index).filter(|known| known.target() == item).map(IssueState::rated))
    }

    #[must_use]
    pub fn open_items(&self) -> Vec<RatedItem> {
        let concerns = self.concerns.iter().map(ConcernState::target);
        let targets = concerns.chain(self.issues.iter().map(IssueState::target));
        targets.filter(|item| self.closed(item.entry()).is_none()).filter_map(|item| self.rated_item(item)).collect()
    }

    #[must_use]
    pub fn closed(&self, entry: Entry) -> Option<()> {
        let addressed = entry.target().and_then(|target| self.addressed_by(target));
        let done = entry.task().and_then(|task| self.task(task)).filter(|task| task.progress().done().is_some());
        addressed.map(|_| ()).or(done.map(|_| ())).or(self.retirement(entry).map(|_| ()))
    }

    #[must_use]
    pub fn retirement(&self, entry: Entry) -> Option<&Retired> {
        self.retired.get(&entry)
    }

    #[must_use]
    pub fn concerns(&self) -> &[ConcernState] {
        &self.concerns
    }

    #[must_use]
    pub fn issues(&self) -> &[IssueState] {
        &self.issues
    }

    #[must_use]
    pub fn tasks(&self) -> &[TaskState] {
        &self.tasks
    }

    pub(crate) fn anomalies(&self) -> &[Anomaly] {
        &self.anomalies
    }

    pub(crate) fn addressed_by(&self, target: Target) -> Option<Id<Task>> {
        self.addressers.get(&target)?.iter().find_map(|task| self.task(*task)?.addressing(target))
    }

    fn task(&self, task: Id<Task>) -> Option<&TaskState> {
        self.index_of.get(&Entry::Task(task)).and_then(|index| self.tasks.get(*index))
    }

    fn known(&self, entry: Entry) -> Result<(), Refusal> {
        self.index_of.contains_key(&entry).then_some(()).ok_or(Refusal::UnknownEntry(entry))
    }

    fn fresh(&self, target: Target) -> Result<(), Refusal> {
        self.known(target.entry()).map_or(Ok(()), |()| Err(Refusal::DuplicateId))
    }
}

#[cfg(test)]
mod tests;
