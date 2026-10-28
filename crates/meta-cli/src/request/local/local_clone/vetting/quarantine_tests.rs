use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use meta_run::{CommandLine, Environment};

use super::deletable::Deletable;
use super::entry_kind::EntryKind;
use super::fake_host::{FakeHost, PATH, TASK};
use super::git_call::GitCall;
use super::vetted::Vetted;
use crate::request::local::recorded_worktree::RecordedWorktree;

const LATE: &str = "late.log";
const RACES: [[&str; 2]; 2] = [["status", "?? late.log\0"], ["rev-parse", "c5\n"]];
const RACED: [&str; 2] = ["untracked files", "changed during checks"];
const QUARANTINE: &str = "/trees/.gc-task-00000000000000b1-1x0";
const MOVED_OUT: &str = "worktree move /trees/task-00000000000000b1 /trees/.gc-task-00000000000000b1";
const MOVED_BACK: &str = "worktree move /trees/.gc-task-00000000000000b1";
const RESTORED: &str = " /trees/task-00000000000000b1";
const REMOVE: &str = "worktree remove";
const FROZEN: &str = "frozen /trees/.gc-task-00000000000000b1-1x0";
const THAWED: &str = "thawed /trees/.gc-task-00000000000000b1-1x0";
const INHERITED: [(&str, &str); 3] = [("GIT_DIR", "d"), ("HOME", "h"), ("GIT_INDEX_FILE", "i")];
const ISOLATED: [(&str, Option<&str>); 5] = [
    ("GIT_CONFIG_GLOBAL", Some("/dev/null")),
    ("GIT_CONFIG_NOSYSTEM", Some("1")),
    ("GIT_CONFIG_SYSTEM", Some("/dev/null")),
    ("GIT_DIR", None),
    ("GIT_INDEX_FILE", None),
];
const STAT_PINS: [&str; 2] = ["core.trustctime=true", "core.checkStat=default"];
const REFUSALS: [&str; 4] = ["no recorded worktree id", "changed during checks", "in use", "in use"];

struct Sweep(FakeHost);

impl Sweep {
    fn refusal(&self) -> Option<String> {
        let vet = || Vetted::proved(&FakeHost::worktree(), &self.0.context(), &self.0);
        let deletable = Deletable::confirmed(vet().unwrap(), vet());
        let removed =
            deletable.and_then(|deletable| deletable.removed(&FakeHost::worktree(), &self.0.context(), &self.0));
        removed.err().map(|kept| kept.to_string())
    }
}

#[test]
fn a_file_or_commit_made_after_the_vet_is_caught_in_quarantine_and_the_worktree_moves_back() {
    let sweeps = RACES.map(|race| {
        let mut host = FakeHost::clean();
        host.races(race);
        Sweep(host)
    });
    assert_eq!(sweeps.each_ref().map(Sweep::refusal), RACED.map(|reason| Some(reason.to_owned())));
    let calls: Vec<Vec<String>> = sweeps.iter().map(|sweep| sweep.0.calls()).collect();
    assert!(
        calls.iter().all(|calls| {
            calls.iter().any(|call| call.contains(MOVED_OUT))
                && calls.last().is_some_and(|call| call.contains(MOVED_BACK) && call.ends_with(RESTORED))
                && !calls.iter().any(|call| call.contains(REMOVE))
                && [FROZEN, THAWED].iter().all(|step| calls.iter().any(|call| call == step))
        }),
        "{calls:?}"
    );
}

#[test]
fn vetting_git_ignores_inherited_git_variables_and_global_and_system_config_and_pins_stat_checks() {
    let call = GitCall::common(Path::new(QUARANTINE), &[OsStr::new(LATE)]);
    let inherited = INHERITED.map(|(name, value)| (OsString::from(name), Some(OsString::from(value))));
    let run = call.run(Environment::from(BTreeMap::from(inherited)));
    let isolated = ISOLATED.map(|(name, value)| (OsString::from(name), value.map(OsString::from)));
    assert_eq!(Environment::from(&run), Environment::from(BTreeMap::from(isolated)));
    let line = CommandLine::from(&run).to_string();
    assert!(STAT_PINS.iter().all(|pin| line.split(' ').any(|argument| argument == *pin)));
}

#[test]
fn an_unidentified_record_or_an_occupied_quarantine_keeps_the_worktree() {
    let host = FakeHost::clean();
    let unidentified = RecordedWorktree::new(PathBuf::from(PATH), TASK.parse().unwrap());
    let refused = Vetted::proved(&unidentified, &host.context(), &host).err().map(|kept| kept.to_string());
    let mut occupied = FakeHost::clean();
    occupied.entry_is(QUARANTINE, EntryKind::Dir);
    let mut followed = FakeHost::clean();
    followed.opens(PathBuf::from(QUARANTINE).join(LATE));
    let followed = Sweep(followed);
    let mut busy = FakeHost::clean();
    busy.opens(PathBuf::from(PATH).join(LATE));
    let busy = Sweep(busy);
    let refusals = [refused, Sweep(occupied).refusal(), followed.refusal(), busy.refusal()];
    assert_eq!(refusals, REFUSALS.map(|reason| Some(reason.to_owned())));
    assert!(followed.0.calls().last().is_some_and(|call| call.contains(MOVED_BACK) && call.ends_with(RESTORED)));
    assert!(!busy.0.calls().iter().any(|call| call.contains(MOVED_OUT)));
}
