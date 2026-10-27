use meta_log::Snapshot;
use meta_schema::{Id, Task};
use task::BranchName;

use crate::task::ci::run_kind::RunKind;
use crate::task::failure::Failure;

pub struct TrackedWork;

impl TrackedWork {
    pub fn refuse_untracked(run: &RunKind, read: impl FnOnce() -> Result<Snapshot, Failure>) -> Result<(), Failure> {
        let _ = (run, Self::named);
        read().map(drop)
    }

    fn named(branch: &BranchName) -> Option<Id<Task>> {
        let _ = branch;
        None
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

    const LOG: [&str; 9] = [
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000a1","text":"x","division":"features-and-maintenance"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskClaimed","task":"task-00000000000000a1"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000b2","text":"x","division":"features-and-maintenance"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000c3","text":"x","division":"features-and-maintenance"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskClaimed","task":"task-00000000000000c3"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"EntryRetired","entry":"task-00000000000000c3","text":"x"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000d4","text":"x","division":"features-and-maintenance"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskClaimed","task":"task-00000000000000d4"}"#,
        r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskDone","task":"task-00000000000000d4"}"#,
    ];
    const REFUSED: [&str; 4] = ["feature-x", "task-00000000000000e5", "task-00000000000000c3", "task-00000000000000b2"];
    const PASSED: [&str; 3] = ["task-00000000000000a1", "task-00000000000000a1-rated", "task-00000000000000d4"];
    const REMEDY: [&str; 3] = ["meta open \"", "--division ", "meta claim <task> --worktree"];

    impl TrackedWork {
        fn judged(branch: Option<&str>) -> Result<(), Failure> {
            let parsed = |line: &&str| serde_json::from_str::<Record>(line).expect("record");
            let log = || Ok(LOG.iter().fold(Snapshot::default(), |log, line| log.with_record(parsed(line))));
            let Ok(branch) = branch.map(str::parse).transpose();
            TrackedWork::refuse_untracked(&RunKind::Pull(branch), log)
        }
    }

    #[test]
    fn no_branch_no_task_unknown_retired_or_unclaimed_tasks_and_an_unreadable_log_fail_with_the_remedy() {
        let [plain, unknown, retired, unclaimed] = REFUSED.map(|branch| TrackedWork::judged(Some(branch)));
        let named = |task: &str| task.parse().expect("task id");
        let expected = [
            Untracked::NoBranch,
            Untracked::NotTask(REFUSED[0].parse().expect("branch")),
            Untracked::Unknown(named(REFUSED[1])),
            Untracked::Retired(named(REFUSED[2])),
            Untracked::Unclaimed(named(REFUSED[3])),
        ]
        .map(|reason| Failure::Untracked(reason).to_string());
        let found = [TrackedWork::judged(None), plain, unknown, retired, unclaimed]
            .map(|refused| refused.err().map(|failure| failure.to_string()));
        assert!(found.iter().zip(&expected).all(|(found, expected)| found.as_ref() == Some(expected)));
        assert!(expected.iter().all(|shown| REMEDY.iter().all(|step| shown.contains(step))));
        let unreadable = || Err(Failure::StandingLog(LogFailure::Contended));
        let refused = TrackedWork::refuse_untracked(&RunKind::Pull(PASSED[0].parse().ok()), unreadable);
        assert!(matches!(refused, Err(Failure::Untracked(Untracked::Unreadable(_)))));
    }

    #[test]
    fn a_claimed_or_done_task_branch_with_or_without_a_suffix_passes_and_a_run_outside_a_pull_request_reads_nothing() {
        assert!(PASSED.iter().all(|branch| TrackedWork::judged(Some(branch)).is_ok()));
        let unreadable = || Err(Failure::StandingLog(LogFailure::Contended));
        assert!(TrackedWork::refuse_untracked(&RunKind::Other, unreadable).is_ok());
        assert!(TrackedWork::named(&PASSED[1].parse().expect("branch")).is_some());
    }
}
