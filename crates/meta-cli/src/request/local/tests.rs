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
const TRAIL_DIR: &str = "meta-local";
const TRAIL: &str = "meta-local/worktrees.jsonl";
const TORN: &str = "{\"change\":\"Crea";
const WARNING: &str = "warning: skipped a torn record line";

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
    assert!(absent.identity().is_err() && LocalTrail::read(Path::new(NOWHERE)).unwrap().current().is_empty());
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
        TrailEntry::Created(RecordedWorktree::new(here.clone(), task)),
    ]);
    let shown = Worktrees::joined(trail, &[here.clone(), PathBuf::from(STRAY)], &state).to_string();
    let kept = format!("{task} assigned {task} {} created", here.display());
    assert!(shown.matches(&kept).count() == 1 && !shown.contains(DROPPED));
    assert!(shown.contains(&format!("{} not in the log", gone.task())) && shown.matches(MISSING).count() == 1);
    assert!(shown.ends_with(&format!("{UNRECORDED}{STRAY}\n")) && shown.matches(UNRECORDED).count() == 1);
}

#[test]
fn appends_after_a_torn_line_start_a_new_line_and_reads_skip_the_torn_line_with_a_warning() {
    let git_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(git_dir.path().join(TRAIL_DIR)).unwrap();
    std::fs::write(git_dir.path().join(TRAIL), TORN).unwrap();
    let created =
        [GONE, STRAY].map(|path| TrailEntry::Created(RecordedWorktree::new(PathBuf::from(path), Id::generate())));
    assert!(created.iter().all(|entry| LocalTrail::append(git_dir.path(), entry).is_ok()));
    let trail = LocalTrail::read(git_dir.path()).unwrap();
    let shown = Worktrees::joined(trail, &[], &Fixture::assigned().state).to_string();
    assert!(shown.starts_with(WARNING) && shown.matches(WARNING).count() == 1);
    assert_eq!(shown.lines().count(), created.len() + 1);
}
