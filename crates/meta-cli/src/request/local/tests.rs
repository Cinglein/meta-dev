use std::path::{Path, PathBuf};

use meta_schema::{Actor, Claim, Event, Id, NewTask, Record, Task};
use meta_state::State;

use super::local_settings::LocalSettings;
use super::local_trail::LocalTrail;
use super::recorded_worktree::RecordedWorktree;
use super::trail_entry::TrailEntry;
use super::worktrees::Worktrees;

const CHECKOUT: &str = "/work/clone";
const SIBLING: &str = "/work/clone-worktrees";
const RELATIVE: &str = "worktree_root = \"trees\"";
const NESTED: &str = "/work/clone/trees";
const NOWHERE: &str = "/nonexistent/clone";
const NOT_A_DIRECTORY: &str = "/dev/null";
const REFUSED: [&str; 2] = ["[identity]\nname = \"a\"\nemail = \"a@example.com\"", "root = \"trees\""];
const ACTOR: &str = "agent";
const DIVISION: &str = "features-and-maintenance";
const GONE: &str = "/nonexistent/gone";
const DROPPED: &str = "/nonexistent/dropped";
const STRAY: &str = "/nonexistent/stray";
const MISSING: &str = " missing";
const UNRECORDED: &str = "unrecorded ";

struct Fixture {
    state: State,
    task: Id<Task>,
}

impl Fixture {
    fn assigned() -> Self {
        let opened = NewTask::new(DIVISION.parse().unwrap(), ACTOR.parse().unwrap(), vec![]);
        let task = opened.task();
        let actor: Actor = ACTOR.parse().unwrap();
        let claimed = Record::new(actor.clone(), Event::TaskClaimed(Claim::new(task, None)));
        Self { state: State::fold(&[Record::new(actor, Event::TaskOpened(opened)), claimed]), task }
    }

    fn trail(entries: &[TrailEntry]) -> LocalTrail {
        let lines: Vec<String> = entries.iter().map(|entry| serde_json::to_string(entry).unwrap()).collect();
        lines.join("\n").parse().unwrap()
    }
}

#[test]
fn settings_default_to_a_sibling_root_resolve_relative_roots_and_refuse_foreign_emails() {
    let checkout = Path::new(CHECKOUT);
    let absent = LocalSettings::read(Path::new(NOWHERE)).unwrap();
    let relative: LocalSettings = RELATIVE.parse().unwrap();
    let roots = [absent.worktree_root(checkout), relative.worktree_root(checkout)];
    assert_eq!(roots, [PathBuf::from(SIBLING), PathBuf::from(NESTED)]);
    assert!(absent.identity().is_err() && LocalTrail::read(Path::new(NOWHERE)).unwrap() == LocalTrail::default());
    let unreadable = LocalSettings::read(Path::new(NOT_A_DIRECTORY)).unwrap_err().to_string();
    assert!(REFUSED.iter().all(|text| text.parse::<LocalSettings>().is_err()) && unreadable.contains(NOT_A_DIRECTORY));
}

#[test]
fn worktrees_join_the_current_trail_to_task_status_and_flag_missing_and_unrecorded_paths() {
    let Fixture { state, task } = Fixture::assigned();
    let here = std::env::temp_dir();
    let gone = RecordedWorktree::new(PathBuf::from(GONE), Id::generate());
    let trail = Fixture::trail(&[
        TrailEntry::Created(RecordedWorktree::new(here.clone(), task)),
        TrailEntry::Created(RecordedWorktree::new(PathBuf::from(DROPPED), task)),
        TrailEntry::Created(gone.clone()),
        TrailEntry::Removed { path: PathBuf::from(DROPPED), at: chrono::Utc::now() },
    ]);
    let shown = Worktrees::joined(&trail, &[here.clone(), PathBuf::from(STRAY)], &state).to_string();
    assert!(shown.contains(&format!("{task} assigned {task} {} created", here.display())) && !shown.contains(DROPPED));
    assert!(shown.contains(&format!("{} not in the log", gone.task())) && shown.matches(MISSING).count() == 1);
    assert!(shown.ends_with(&format!("{UNRECORDED}{STRAY}\n")) && shown.matches(UNRECORDED).count() == 1);
}
