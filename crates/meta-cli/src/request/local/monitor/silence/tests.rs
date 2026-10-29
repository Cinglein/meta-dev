use chrono::{DateTime, TimeDelta, Utc};
use meta_schema::{Actor, Claim, Event, NewTask, Record, TaskRef};
use meta_state::State;
use tempfile::TempDir;

use super::{Hush, Silence};
use crate::request::ci::SessionDir;
use crate::request::cli_failure::CliFailure;
use crate::request::hook::CallPhase;
use crate::request::local::local_trail::LocalTrail;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::trail_entry::TrailEntry;

const TASKS: [&str; 4] =
    ["task-00000000000000c0", "task-00000000000000c1", "task-00000000000000c2", "task-00000000000000c3"];
const OPENED: [&str; 4] = [
    r#"{"task":"task-00000000000000c0","text":"t"}"#,
    r#"{"task":"task-00000000000000c1","text":"t"}"#,
    r#"{"task":"task-00000000000000c2","text":"t"}"#,
    r#"{"task":"task-00000000000000c3","text":"t"}"#,
];
const CALLS: [Option<CallPhase>; 4] = [Some(CallPhase::Finished), Some(CallPhase::Started), None, None];
const ACTOR: &str = "alice";
const GIT: &str = ".git";
const LINK: &str = "gitdir: ../.git\n";
const NOW: &str = "2026-10-29T12:00:00Z";
const POLICIES: [&str; 2] = ["silent_after = 1800000\nlong_call_after = 1800000", ""];
const LATER: TimeDelta = TimeDelta::minutes(31);
const HEARD: [&str; 3] = [
    "silent task-00000000000000c0: no tool call for 1860s, limit 1800s",
    "long call task-00000000000000c1: one tool call running 1860s, limit 1800s, report only",
    "unstamped task-00000000000000c2: live worktree with no tool-call stamp",
];
const SILENCES: [&str; 2] = [
    r#"{"task":"task-00000000000000c0","worktree":"/w/a","hush":"Unstamped"}"#,
    r#"{"task":"task-00000000000000c1","worktree":"/w/a","hush":"Unstamped"}"#,
];

struct Scene {
    root: TempDir,
    at: DateTime<Utc>,
}

impl Scene {
    fn new() -> Self {
        let scene = Self { root: tempfile::tempdir().unwrap(), at: NOW.parse().unwrap() };
        std::fs::create_dir(scene.root.path().join(GIT)).unwrap();
        TASKS.iter().zip(CALLS).for_each(|(task, call)| {
            let worktree = scene.root.path().join(task);
            std::fs::create_dir(&worktree).and_then(|()| std::fs::write(worktree.join(GIT), LINK)).unwrap();
            let created = TrailEntry::Created(RecordedWorktree::new(worktree.clone(), task.parse().unwrap()));
            LocalTrail::append(&scene.root.path().join(GIT), &created).unwrap();
            call.and_then(|call| SessionDir::stamped(&worktree, call, scene.at)).transpose().unwrap();
        });
        scene
    }

    fn state() -> State {
        let opened = OPENED.map(|text| Event::TaskOpened(serde_json::from_str::<NewTask>(text).unwrap()));
        let done = TASKS[3].parse().unwrap();
        let finished = [Event::TaskClaimed(Claim::new(done, None)), Event::TaskDone(TaskRef::new(done))];
        let events = opened.into_iter().chain(finished);
        let records: Vec<Record> = events.map(|event| Record::new(ACTOR.parse::<Actor>().unwrap(), event)).collect();
        State::fold(&records)
    }

    fn heard(&self, policy: &str) -> Vec<String> {
        let git_dir = self.root.path().join(GIT).canonicalize().unwrap();
        let found = Silence::heard(
            &git_dir,
            toml::from_str(policy).unwrap(),
            || Ok::<_, CliFailure>(Self::state()),
            self.at + LATER,
        );
        found.unwrap().iter().map(ToString::to_string).collect()
    }
}

#[test]
fn active_worktrees_quiet_past_the_limit_running_long_or_unstamped_are_heard_and_done_tasks_are_not() {
    let scene = Scene::new();
    let [limited, unlimited] = POLICIES;
    assert_eq!(scene.heard(limited), HEARD);
    assert_eq!(scene.heard(unlimited), Vec::<String>::new());
}

#[test]
fn a_silence_repeats_only_one_of_the_same_task_and_a_quiet_spell_at_the_limit_is_not_silent() {
    let [first, other] = SILENCES.map(|text| serde_json::from_str::<Silence>(text).unwrap());
    assert_eq!((first.same(&first), first.same(&other)), (Some(()), None));
    assert_eq!(Hush::timed(CallPhase::Finished, LATER.to_std().unwrap(), LATER.to_std().unwrap()), None);
}
