use meta_log::Snapshot;
use meta_schema::PullRequest;
use meta_state::TaskStatus;

use crate::task::ci::head_branch::HeadBranch;
use crate::task::ci::run_kind::RunKind;
use crate::task::ci::standing_log::StandingLog;
use crate::task::ci::untracked::Untracked;
use crate::task::failure::Failure;

pub struct TrackedWork;

impl TrackedWork {
    pub fn refuse_untracked(run: &RunKind, read: impl FnOnce() -> Result<Snapshot, Failure>) -> Result<(), Failure> {
        run.refuse(|head, pull| Self::claimed(head, pull, read))
    }

    fn claimed(
        head: Option<&HeadBranch>,
        pull: Option<PullRequest>,
        read: impl FnOnce() -> Result<Snapshot, Failure>,
    ) -> Result<(), Untracked> {
        let task = head.ok_or(Untracked::NoBranch)?.task()?;
        let state = StandingLog::state(read()).map_err(|failure| Untracked::Unreadable(Box::new(failure)))?;
        let known =
            state.tasks().iter().find(|known| known.new_task().task() == task).ok_or(Untracked::Unknown(task))?;
        let progress = known.progress();
        let retired = (state.status_of(known) == TaskStatus::Retired).then_some(Untracked::Retired(task));
        let unclaimed =
            (progress.done().is_none() && progress.assignments().is_empty()).then_some(Untracked::Unclaimed(task));
        let reused = (progress.done().is_some() && !pull.is_some_and(|pull| known.pull_requests().contains(&pull)))
            .then_some(Untracked::DoneElsewhere(task));
        retired.or(unclaimed).or(reused).map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use meta_golden::Golden;
    use meta_log::{LogFailure, Snapshot};
    use meta_schema::Record;

    use super::TrackedWork;
    use crate::task::ci::run_kind::RunKind;
    use crate::task::failure::Failure;

    const LOG: &str = "TaskOpened a1 TaskClaimed a1 TaskOpened b2 TaskOpened c3 EntryRetired c3 TaskOpened d4 TaskClaimed d4 TaskLinked d4 TaskDone d4";
    const RECORD: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"KIND","task":"task-00000000000000ID","entry":"task-00000000000000ID","text":"x","division":"features-and-maintenance","pr":7}"#;
    const NUMBER: &str = "7";
    const LINKED: (&str, &str) = ("PULL_REQUEST", NUMBER);
    const UNLINKED: (&str, &str) = ("PULL_REQUEST", "8");
    const REFUSED: [&str; 4] = ["feature-x", "task-00000000000000e5", "task-00000000000000c3", "task-00000000000000b2"];
    const DONE: &str = "task-00000000000000d4";
    const PASSED: [&str; 3] = ["task-00000000000000a1-rated", "task-00000000000000a1/rated", DONE];
    const GATE: (&str, &str) = ("META_GATE_BRANCH", PASSED[1]);
    const PUSH: [(&str, &str); 2] = [("GITHUB_EVENT_NAME", "push"), ("GITHUB_HEAD_REF", "")];
    const SIGNALS: [(&str, &str); 4] =
        [("GITHUB_EVENT_NAME", "pull_request_target"), LINKED, (GATE.0, ""), (PUSH[1].0, GATE.1)];

    impl TrackedWork {
        pub(crate) fn judged(branch: Option<&str>, pull: (&str, &str)) -> Result<(), Failure> {
            let words: Vec<&str> = LOG.split(' ').collect();
            let line = |pair: &[&str]| RECORD.replace("KIND", pair[0]).replace("ID", pair[1]);
            let log = words.chunks(2).map(|pair| serde_json::from_str::<Record>(&line(pair)).unwrap());
            let snapshot = || Ok(log.fold(Snapshot::default(), Snapshot::with_record));
            TrackedWork::refuse_untracked(&RunKind::seen(&[(GATE.0, branch.unwrap_or_default()), pull]), snapshot)
        }

        fn unreadable() -> Result<Snapshot, Failure> {
            Err(Failure::StandingLog(LogFailure::Contended))
        }
    }

    impl RunKind {
        fn seen(pairs: &[(&str, &str)]) -> RunKind {
            RunKind::from_ambient(&pairs.iter().map(|(name, value)| ((*name).into(), (*value).into())).collect())
        }
    }

    #[test]
    fn no_branch_no_task_unknown_retired_or_unclaimed_tasks_and_an_unreadable_log_fail_with_the_remedy() {
        let found = [None].into_iter().chain(REFUSED.map(Some)).map(|branch| TrackedWork::judged(branch, LINKED));
        let unreadable =
            TrackedWork::refuse_untracked(&RunKind::Pull(PASSED[0].parse().ok(), None), TrackedWork::unreadable);
        let refused = found.chain([unreadable]).map(|refused| refused.err().map(|failure| failure.to_string()));
        Golden::for_test(module_path!(), "refused").debugged(&refused.collect::<Vec<_>>()).unwrap();
    }

    #[test]
    fn claimed_or_done_task_branches_pass_runs_without_a_pull_request_signal_read_nothing_and_any_signal_is_a_pull() {
        let unread = TrackedWork::refuse_untracked(&RunKind::Other, TrackedWork::unreadable).is_ok();
        let reused = TrackedWork::judged(Some(DONE), UNLINKED).is_err();
        assert!(unread && reused && PASSED.iter().all(|branch| TrackedWork::judged(Some(branch), LINKED).is_ok()));
        let [event, number, gate, head] = SIGNALS.map(|signal| RunKind::seen(&[signal]));
        let (skipped, unnamed) = ([RunKind::seen(&[]), RunKind::seen(&PUSH)], RunKind::Pull(None, None));
        assert!(skipped.iter().all(|kind| *kind == RunKind::Other) && head == RunKind::seen(&[GATE]));
        let numbered = number == RunKind::Pull(None, NUMBER.parse().ok());
        assert!([event, gate].iter().all(|kind| *kind == unnamed) && numbered && head != unnamed);
    }
}
