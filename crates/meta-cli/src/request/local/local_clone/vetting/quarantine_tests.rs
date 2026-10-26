use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::deletable::Deletable;
use super::entry_kind::EntryKind;
use super::fake_host::{FakeHost, PATH, TASK};
use super::git_call::GitCall;
use super::occupancy::Occupancy;
use super::vetted::Vetted;
use crate::request::local::recorded_worktree::RecordedWorktree;

const LATE: &str = "late.log";
const RACED: &str = "untracked files";
const QUARANTINE: &str = "/trees/.gc-task-00000000000000b1";
const MOVED_OUT: &str = "worktree move /trees/task-00000000000000b1 /trees/.gc-task-00000000000000b1";
const MOVED_BACK: &str = "worktree move /trees/.gc-task-00000000000000b1 /trees/task-00000000000000b1";
const REMOVE: &str = "worktree remove";
const INHERITED: [&str; 3] = ["GIT_DIR", "HOME", "GIT_INDEX_FILE"];
const ISOLATED: [(&str, Option<&str>); 5] = [
    ("GIT_CONFIG_GLOBAL", Some("/dev/null")),
    ("GIT_CONFIG_NOSYSTEM", Some("1")),
    ("GIT_CONFIG_SYSTEM", Some("/dev/null")),
    ("GIT_DIR", None),
    ("GIT_INDEX_FILE", None),
];
const STAT_PINS: [&str; 2] = ["core.trustctime=true", "core.checkStat=default"];
const REFUSALS: [&str; 2] = ["no recorded worktree id", "changed during checks"];

struct Sweep(FakeHost);

impl Sweep {
    fn refusal(&self) -> Option<String> {
        let vet = || Vetted::proved(&FakeHost::worktree(), &self.0.context(), &self.0);
        let deletable = Deletable::confirmed(vet().unwrap(), &Occupancy::Listed(vec![]), vet());
        deletable.and_then(|deletable| deletable.removed(&self.0)).err().map(|kept| kept.to_string())
    }
}

#[test]
fn a_file_written_after_the_vet_is_caught_in_quarantine_and_the_worktree_moves_back() {
    let mut host = FakeHost::clean();
    host.races(LATE);
    let sweep = Sweep(host);
    assert_eq!(sweep.refusal(), Some(RACED.to_owned()));
    let calls = sweep.0.calls();
    assert!(
        calls.iter().any(|call| call.ends_with(MOVED_OUT))
            && calls.last().is_some_and(|call| call.ends_with(MOVED_BACK))
    );
    assert!(!calls.iter().any(|call| call.contains(REMOVE)), "{calls:?}");
}

#[test]
fn vetting_git_ignores_inherited_git_variables_and_global_and_system_config_and_pins_stat_checks() {
    let call = GitCall::common(Path::new(QUARANTINE), &[OsStr::new(LATE)]);
    let command = call.command(INHERITED.map(OsString::from));
    let envs: HashMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
    let expected: HashMap<&OsStr, Option<&OsStr>> =
        ISOLATED.iter().map(|(name, value)| (OsStr::new(*name), value.map(OsStr::new))).collect();
    assert_eq!(envs, expected);
    assert!(STAT_PINS.iter().all(|pin| command.get_args().any(|argument| argument == *pin)));
}

#[test]
fn an_unidentified_record_or_an_occupied_quarantine_keeps_the_worktree() {
    let host = FakeHost::clean();
    let unidentified = RecordedWorktree::new(PathBuf::from(PATH), TASK.parse().unwrap());
    let refused = Vetted::proved(&unidentified, &host.context(), &host).err().map(|kept| kept.to_string());
    let mut occupied = FakeHost::clean();
    occupied.entry_is(QUARANTINE, EntryKind::Dir);
    assert_eq!([refused, Sweep(occupied).refusal()], REFUSALS.map(|reason| Some(reason.to_owned())));
}
