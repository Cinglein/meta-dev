use std::fmt;

use meta_schema::Tag;

use crate::state::State;
use crate::task_state::TaskState;

#[derive(Debug)]
pub struct Summary<'state> {
    state: &'state State,
}

impl<'state> Summary<'state> {
    pub fn new(state: &'state State) -> Self {
        Self { state }
    }
}

impl fmt::Display for Summary<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state;
        let unfinished: Vec<&TaskState> =
            state.tasks().iter().filter(|task| task.status().unfinished().is_some()).collect();
        let claimed = unfinished.iter().filter(|task| task.status().claimant().is_some());
        let open = unfinished.iter().filter(|task| task.status().claimant().is_none());
        writeln!(formatter, "Tasks: {} unfinished, {} done", unfinished.len(), state.tasks().len() - unfinished.len())?;
        claimed.chain(open).try_for_each(|task| writeln!(formatter, "  {task}"))?;
        let issues: Vec<_> =
            state.issues().iter().filter(|issue| state.addressed_by(issue.target()).is_none()).collect();
        writeln!(formatter, "Issues: {} open", issues.len())?;
        issues.iter().try_for_each(|issue| writeln!(formatter, "  {issue}"))?;
        let concerns: Vec<_> =
            state.concerns().iter().filter(|concern| state.addressed_by(concern.target()).is_none()).collect();
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
