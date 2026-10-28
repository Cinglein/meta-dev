use std::collections::BTreeMap;
use std::ffi::OsString;

use meta_schema::PullRequest;
use task::ChangeLink;

use crate::task::ci::head_branch::HeadBranch;
use crate::task::ci::untracked::Untracked;
use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunKind {
    Pull(Option<HeadBranch>, Option<PullRequest>),
    Other,
}

impl RunKind {
    pub const GATE_BRANCH: &str = "META_GATE_BRANCH";
    const EVENT: &str = "GITHUB_EVENT_NAME";
    const PULL_EVENT: &str = "pull_request";
    const HEAD: &str = "GITHUB_HEAD_REF";
    pub const NUMBER: &str = "PULL_REQUEST";

    pub fn from_ambient(ambient: &BTreeMap<String, OsString>) -> RunKind {
        let set = |name: &str| ambient.get(name).map(|value| value.to_string_lossy().into_owned());
        let given = |name: &str| set(name).filter(|value| !value.is_empty());
        let head = given(Self::HEAD).or_else(|| given(Self::GATE_BRANCH));
        let pulled = given(Self::EVENT).filter(|event| event.starts_with(Self::PULL_EVENT));
        let number = given(Self::NUMBER);
        let signal = pulled.or(number.clone()).or(set(Self::GATE_BRANCH)).or(head.clone());
        let Ok(branch) = head.map(|head| head.parse()).transpose();
        let pull = number.and_then(|number| number.parse().ok());
        signal.map_or(RunKind::Other, |_| RunKind::Pull(branch, pull))
    }

    pub fn link(&self) -> ChangeLink {
        let task = match self {
            RunKind::Pull(Some(head), _) => head.task().ok(),
            RunKind::Pull(None, _) | RunKind::Other => None,
        };
        ChangeLink::new(None, task)
    }

    pub fn by_kind<T, R>(&self, target: &mut T, pull: impl FnOnce(&mut T) -> R, other: impl FnOnce(&mut T) -> R) -> R {
        match self {
            RunKind::Pull(..) => pull(target),
            RunKind::Other => other(target),
        }
    }

    pub fn refuse(
        &self,
        judge: impl FnOnce(Option<&HeadBranch>, Option<PullRequest>) -> Result<(), Untracked>,
    ) -> Result<(), Failure> {
        match self {
            RunKind::Pull(head, pull) => judge(head.as_ref(), *pull).map_err(Failure::Untracked),
            RunKind::Other => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use meta_log::Snapshot;
    use meta_schema::Record;

    use super::RunKind;
    use crate::task::ci::standing_log::StandingLog;

    const RECORD: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000ID","entry":"task-00000000000000ID","text":"x","division":"DIVISION"}"#;
    const OPENED: [(&str, &str); 2] = [("a1", "performance-and-cost"), ("b2", "features-and-maintenance")];
    const RUNS: [(&str, &str); 2] =
        [("GITHUB_HEAD_REF", "task-00000000000000a1-2"), ("META_GATE_BRANCH", "task-00000000000000a1/1")];

    #[test]
    fn a_stacked_branch_and_a_local_gate_run_link_their_own_tasks_division() {
        let record = |(id, division): (&str, &str)| RECORD.replace("ID", id).replace("DIVISION", division);
        let records = OPENED.map(|opened| serde_json::from_str::<Record>(&record(opened)).unwrap());
        let state =
            StandingLog::state(Ok(records.into_iter().fold(Snapshot::default(), Snapshot::with_record))).unwrap();
        let linked = RUNS.map(|(name, value)| {
            let link = RunKind::from_ambient(&[(name.into(), value.into())].into_iter().collect()).link();
            let tasks = state.tasks().iter().filter(|task| link.links(task));
            tasks.filter_map(|task| task.new_task().placement().division().map(ToString::to_string)).collect::<Vec<_>>()
        });
        assert_eq!(linked, [[OPENED[0].1], [OPENED[0].1]]);
    }
}
