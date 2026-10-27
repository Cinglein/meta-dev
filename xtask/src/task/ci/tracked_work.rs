use meta_log::Snapshot;
use meta_state::TaskStatus;

use crate::task::ci::head_branch::HeadBranch;
use crate::task::ci::run_kind::RunKind;
use crate::task::ci::standing_log::StandingLog;
use crate::task::ci::untracked::Untracked;
use crate::task::failure::Failure;

pub struct TrackedWork;

impl TrackedWork {
    pub fn refuse_untracked(run: &RunKind, read: impl FnOnce() -> Result<Snapshot, Failure>) -> Result<(), Failure> {
        run.refuse(|head| Self::claimed(head, read))
    }

    fn claimed(head: Option<&HeadBranch>, read: impl FnOnce() -> Result<Snapshot, Failure>) -> Result<(), Untracked> {
        let task = head.ok_or(Untracked::NoBranch)?.task()?;
        let state = StandingLog::state(read()).map_err(|failure| Untracked::Unreadable(Box::new(failure)))?;
        let known =
            state.tasks().iter().find(|known| known.new_task().task() == task).ok_or(Untracked::Unknown(task))?;
        let progress = known.progress();
        let retired = (state.status_of(known) == TaskStatus::Retired).then_some(Untracked::Retired(task));
        let unclaimed =
            (progress.done().is_none() && progress.assignments().is_empty()).then_some(Untracked::Unclaimed(task));
        retired.or(unclaimed).map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use meta_log::{LogFailure, Snapshot};
    use meta_schema::Record;

    use super::TrackedWork;
    use crate::task::ci::run_kind::RunKind;
    use crate::task::ci::untracked::Untracked;
    use crate::task::failure::Failure;

    const LOG: &str = "TaskOpened a1 TaskClaimed a1 TaskOpened b2 TaskOpened c3 EntryRetired c3 TaskOpened d4 TaskClaimed d4 TaskDone d4";
    const RECORD: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"KIND","task":"task-00000000000000ID","entry":"task-00000000000000ID","text":"x","division":"features-and-maintenance"}"#;
    const REFUSED: [&str; 4] = ["feature-x", "task-00000000000000e5", "task-00000000000000c3", "task-00000000000000b2"];
    const PASSED: [&str; 3] = ["task-00000000000000a1", "task-00000000000000a1-rated", "task-00000000000000d4"];
    const REMEDY: [&str; 3] = ["meta open \"", "--division ", "meta claim <task> --worktree"];
    const GATE: (&str, &str) = ("META_GATE_BRANCH", "task-00000000000000a1-rated");
    const PUSH: [(&str, &str); 2] = [("GITHUB_EVENT_NAME", "push"), ("GITHUB_HEAD_REF", "")];
    const SIGNALS: [(&str, &str); 4] = [
        ("GITHUB_EVENT_NAME", "pull_request_target"),
        ("PULL_REQUEST", "7"),
        ("META_GATE_BRANCH", ""),
        ("GITHUB_HEAD_REF", "task-00000000000000a1-rated"),
    ];

    impl TrackedWork {
        fn judged(branch: Option<&str>) -> Result<(), Failure> {
            let words: Vec<&str> = LOG.split(' ').collect();
            let line = |pair: &[&str]| RECORD.replace("KIND", pair[0]).replace("ID", pair[1]);
            let log = words.chunks(2).map(|pair| serde_json::from_str::<Record>(&line(pair)).unwrap());
            let snapshot = || Ok(log.fold(Snapshot::default(), Snapshot::with_record));
            TrackedWork::refuse_untracked(&RunKind::Pull(branch.map(|branch| branch.parse().unwrap())), snapshot)
        }

        fn unreadable() -> Result<Snapshot, Failure> {
            Err(Failure::StandingLog(LogFailure::Contended))
        }

        fn shown([plain, unknown, retired, unclaimed]: [&str; 4]) -> Vec<Option<String>> {
            let task = |id: &str| id.parse().unwrap();
            let named = [
                Untracked::Unknown(task(unknown)),
                Untracked::Retired(task(retired)),
                Untracked::Unclaimed(task(unclaimed)),
            ];
            let reasons = [Untracked::NoBranch, Untracked::NotTask(plain.parse().unwrap())].into_iter().chain(named);
            reasons.map(|reason| Some(Failure::Untracked(reason).to_string())).collect()
        }
    }

    impl RunKind {
        fn seen(pairs: &[(&str, &str)]) -> RunKind {
            RunKind::from_ambient(&pairs.iter().map(|(name, value)| ((*name).into(), (*value).into())).collect())
        }
    }

    #[test]
    fn no_branch_no_task_unknown_retired_or_unclaimed_tasks_and_an_unreadable_log_fail_with_the_remedy() {
        let found = [None].into_iter().chain(REFUSED.map(Some)).map(TrackedWork::judged);
        let shown = TrackedWork::shown(REFUSED);
        assert_eq!(found.map(|refused| refused.err().map(|failure| failure.to_string())).collect::<Vec<_>>(), shown);
        assert!(shown.iter().flatten().all(|shown| REMEDY.iter().all(|step| shown.contains(step))));
        let refused = TrackedWork::refuse_untracked(&RunKind::Pull(PASSED[0].parse().ok()), TrackedWork::unreadable);
        assert!(matches!(refused, Err(Failure::Untracked(Untracked::Unreadable(_)))));
    }

    #[test]
    fn claimed_or_done_task_branches_pass_runs_without_a_pull_request_signal_read_nothing_and_any_signal_is_a_pull() {
        let unread = TrackedWork::refuse_untracked(&RunKind::Other, TrackedWork::unreadable).is_ok();
        assert!(unread && PASSED.map(Some).map(TrackedWork::judged).iter().all(Result::is_ok));
        let [event, number, gate, head] = SIGNALS.map(|signal| RunKind::seen(&[signal]));
        let (skipped, unnamed) = ([RunKind::seen(&[]), RunKind::seen(&PUSH)], RunKind::Pull(None));
        assert!(skipped.iter().all(|kind| *kind == RunKind::Other) && head == RunKind::seen(&[GATE]));
        assert!([event, number, gate].iter().all(|kind| *kind == unnamed) && head != unnamed);
    }
}
