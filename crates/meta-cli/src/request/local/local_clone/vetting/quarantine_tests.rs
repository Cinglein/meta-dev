use std::fs::File;
use std::path::Path;
use std::time::Duration;

use meta_run::{Budget, CommandLine, Environment};

use super::deletable::Deletable;
use super::repo::Repo;
use super::scene::{Real, Recorded, TASK};
use super::vet_argv::VetArgv;
use super::vetted::{Pass, Vetted};
use super::world::World;
use crate::request::local::local_clone::quarantine::{Quarantine, Stamp};
use crate::request::local::recorded_worktree::RecordedWorktree;

const LATE: &str = "late.log";
const RACED: [&str; 2] = ["untracked files", "changed during checks"];
const LATE_IN_THE_WORKTREE: [&str; 2] = ["untracked files", "branch differs from the remote"];
const STAMP: &str = "1x0";
const ISOLATED: [(&str, &str); 6] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_CONFIG_SYSTEM", "/dev/null"),
    ("GIT_NO_REPLACE_OBJECTS", "1"),
    ("GIT_DIR", "/clone/.git"),
    ("GIT_TERMINAL_PROMPT", "0"),
];
const STAT_PINS: [&str; 2] = ["core.trustctime", "core.checkStat"];
const REFUSALS: [&str; 3] = ["no recorded worktree id", "changed during checks", "in use"];
const PATIENT: Duration = Duration::from_secs(10);
const WORDS: [&str; 5] = ["git", "ls-files", "-s", "-v", "-z"];

#[test]
fn the_re_read_in_quarantine_catches_a_late_file_or_commit() {
    let first = Recorded::clean().judged().unwrap();
    let mut late_file = Recorded::clean();
    late_file.status = "?? late.log\0";
    let mut late_commit = Recorded::clean();
    late_commit.survey.local = Ok(Recorded::local(super::scene::REFS, "c5"));
    let rechecked = [late_file, late_commit].map(|scene| {
        let refs = scene.survey.local.clone();
        first.rechecked(&scene.settled(), refs).err().map(|kept| kept.to_string()).unwrap_or_default()
    });
    assert_eq!(rechecked, RACED);
}

#[test]
fn a_late_file_or_commit_after_the_sweep_is_caught_by_the_fresh_re_read_before_any_move() {
    let real = Real::new(&[TASK]);
    let context = real.context();
    let worktree = real.worktree(TASK);
    let swept = || Vetted::proved(&worktree, &context, Pass::Swept).unwrap();
    let late_file = swept();
    std::fs::write(worktree.path().join(LATE), LATE).unwrap();
    let file_refusal = Deletable::confirmed(late_file, Vetted::proved(&worktree, &context, Pass::Fresh));
    std::fs::remove_file(worktree.path().join(LATE)).unwrap();
    let late_commit = swept();
    World::run(worktree.path(), &["commit", "-q", "--allow-empty", "-m", "late"]);
    let commit_refusal = Deletable::confirmed(late_commit, Vetted::proved(&worktree, &context, Pass::Fresh));
    let shown = [file_refusal, commit_refusal].map(|refused| refused.err().map(|kept| kept.to_string()).unwrap_or_default());
    assert_eq!(shown, LATE_IN_THE_WORKTREE);
    assert!(worktree.path().exists() && real.branch(TASK));
}

#[test]
fn a_vetted_worktree_moves_to_quarantine_and_is_removed_with_its_branch() {
    let real = Real::new(&[TASK]);
    let context = real.context();
    let worktree = real.worktree(TASK);
    let vetted = Vetted::proved(&worktree, &context, Pass::Fresh).unwrap();
    let stamp: Stamp = STAMP.parse().unwrap();
    let freed = Deletable::confirmed(vetted.clone(), Ok(vetted)).and_then(|it| it.removed(&worktree, &context, &stamp));
    let quarantine = Quarantine::named(worktree.path(), &stamp);
    assert!(freed.is_ok(), "{freed:?}");
    assert!(!worktree.path().exists() && !quarantine.exists() && !real.branch(TASK));
}

#[test]
fn an_unidentified_record_an_occupied_quarantine_or_an_open_file_keeps_the_worktree() {
    let real = Real::new(&[TASK]);
    let context = real.context();
    let worktree = real.worktree(TASK);
    let unidentified = RecordedWorktree::new(worktree.path().to_path_buf(), TASK.parse().unwrap());
    let refused = Vetted::proved(&unidentified, &context, Pass::Fresh).err();
    let stamp: Stamp = STAMP.parse().unwrap();
    let quarantine = Quarantine::named(worktree.path(), &stamp);
    let removal = || {
        let vetted = Vetted::proved(&worktree, &context, Pass::Fresh).unwrap();
        Deletable::confirmed(vetted.clone(), Ok(vetted)).and_then(|it| it.removed(&worktree, &context, &stamp)).err()
    };
    std::fs::create_dir(&quarantine).unwrap();
    let occupied = removal();
    std::fs::remove_dir(&quarantine).unwrap();
    let _held = File::open(worktree.path().join("meta.toml")).unwrap();
    let busy = removal();
    let refusals = [refused, occupied, busy].map(|kept| kept.map(|kept| kept.to_string()).unwrap_or_default());
    assert_eq!(refusals, REFUSALS);
    assert!(worktree.path().exists() && !quarantine.exists() && real.branch(TASK));
}

#[test]
fn vetting_git_sets_its_own_repository_ignores_global_and_system_config_and_pins_stat_checks() {
    let run = Repo::common(Path::new("/clone/.git"), Budget::new(PATIENT)).run(VetArgv::Index, Vec::new());
    let set = Environment::from(&run);
    let shown = format!("{set:?}");
    let isolated = ISOLATED.iter().all(|(name, value)| shown.contains(&format!("{name:?}: Some({value:?})")));
    let pinned = STAT_PINS.iter().all(|pin| shown.contains(pin));
    let line = CommandLine::from(&run);
    assert!(isolated && pinned && line.words() == WORDS, "{shown}");
}
