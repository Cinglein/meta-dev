use std::fmt;

use meta_schema::Tag;

use crate::state::State;
use crate::task_state::{TaskState, TaskStatus};

#[derive(Debug)]
pub struct Summary<'state> {
    state: &'state State,
}

impl<'state> Summary<'state> {
    #[must_use]
    pub fn new(state: &'state State) -> Self {
        Self { state }
    }
}

impl fmt::Display for Summary<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state;
        let retired = state.tasks().iter().filter(|task| state.retirement(task.entry()).is_some()).count();
        let unfinished: Vec<&TaskState> = state
            .tasks()
            .iter()
            .filter(|task| task.status().unfinished().is_some() && state.retirement(task.entry()).is_none())
            .collect();
        let claimed = unfinished.iter().filter(|task| *task.status() != TaskStatus::Open);
        let open = unfinished.iter().filter(|task| *task.status() == TaskStatus::Open);
        let done = state.tasks().iter().filter(|task| task.status().done().is_some()).count();
        writeln!(formatter, "Tasks: {} unfinished, {done} done, {retired} retired", unfinished.len())?;
        claimed.chain(open).try_for_each(|task| writeln!(formatter, "  {task}"))?;
        let issues: Vec<_> =
            state.issues().iter().filter(|issue| state.closed(issue.target().entry()).is_none()).collect();
        writeln!(formatter, "Issues: {} open", issues.len())?;
        issues.iter().try_for_each(|issue| writeln!(formatter, "  {issue}"))?;
        let concerns: Vec<_> =
            state.concerns().iter().filter(|concern| state.closed(concern.target().entry()).is_none()).collect();
        let tagged = concerns.iter().filter_map(|concern| concern.tagged(Tag::AcceptanceCandidate)).count();
        writeln!(
            formatter,
            "Concerns: {} open, {tagged} #{} (meta list concerns)",
            concerns.len(),
            Tag::AcceptanceCandidate
        )?;
        state.anomalies().iter().try_for_each(|anomaly| writeln!(formatter, "Refused: {anomaly}"))
    }
}
