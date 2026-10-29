use std::path::PathBuf;

use chrono::{DateTime, Utc};
use meta_golden::Golden;
use meta_status::{Ending, Lifetime};

use super::{Census, Collection, Sweeper, Tracked};
use crate::request::local::local_clone::vetting::EntryKind;
use crate::request::local::local_clone::vetting::fake_host::{FakeHost, PATH, TASK};
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::sweep::Sweep;
use meta_process_table::Uninspected;

const ESCAPED: &str = "/elsewhere/task-00000000000000b1";
const GONE: &str = "/trees/gone";
const LIVE: &str = "/trees/live";
const STRAY: &str = "/trees/stray";
const LOST: &str = "/trees/lost";
const KEPT_QUARANTINE: &str = "/trees/.gc-kept-2x0";
const LISTED: [&str; 4] = [PATH, STRAY, "/trees/.gc-lost-1x0", "/trees/.gc-old-9x9"];
const SWEPT: [&str; 6] = [PATH, ESCAPED, GONE, LIVE, LOST, KEPT_QUARANTINE];
const AT: &str = "2026-10-27T12:00:00Z";
const REMOVALS: [&str; 2] =
    ["worktree remove /trees/.gc-task-00000000000000b1-1x0", "update-ref -d refs/heads/task-00000000000000b1 c1"];
const REMOTE_REFS: &str = "ls-remote";
const SWEEP_AND_RECHECK_LISTINGS: usize = 2;
const UNINSPECTED: [&str; 2] =
    ["left alone\n1 process could not be inspected\n", "left alone\n2 processes could not be inspected\n"];

struct Scene {
    host: FakeHost,
    census: Census,
}

impl Scene {
    fn new(tracked: &[&str]) -> Self {
        let mut host = FakeHost::clean();
        host.entry_is(ESCAPED, EntryKind::Dir);
        host.entry_is(LIVE, EntryKind::Dir);
        let lifetime = |path: &str| match path {
            LIVE => Lifetime::Open("2".parse().unwrap()),
            _ => Lifetime::Ended(Ending::Merged("1".parse().unwrap())),
        };
        let recorded = |path: &&str| {
            RecordedWorktree::new(PathBuf::from(path), TASK.parse().unwrap()).identified(FakeHost::worktree_id())
        };
        let tracked = tracked.iter().map(|path| Tracked::new(recorded(path), lifetime(path))).collect();
        Self { host, census: Census::new(tracked, LISTED.map(PathBuf::from).to_vec()) }
    }

    fn swept(&self, sweep: Sweep) -> Collection {
        let context = self.host.context();
        Collection::swept(&Sweeper::new(sweep, &context, &self.host), &self.census)
    }

    fn shown(&self, sweep: Sweep) -> String {
        let collection = self.swept(sweep);
        let trailed = collection.trailed(AT.parse::<DateTime<Utc>>().unwrap());
        let trailed: Vec<String> = trailed.iter().map(|entry| serde_json::to_string(entry).unwrap()).collect();
        format!("{collection}{trailed:#?}\nscanned {:?}\n", self.host.scans())
    }
}

#[test]
fn a_dry_run_vets_and_scans_once_but_deletes_and_trails_nothing() {
    let scene = Scene::new(&SWEPT);
    Golden::for_test(module_path!(), "planned").checked(&scene.shown(Sweep::Planned)).unwrap();
    assert!(!scene.host.calls().iter().any(|call| REMOVALS.iter().any(|removal| call.ends_with(removal))));
}

#[test]
fn a_sweep_re_vets_removes_the_worktree_then_its_branch_and_trails_each_removal_refusal_and_closure() {
    let scene = Scene::new(&SWEPT);
    Golden::for_test(module_path!(), "done").checked(&scene.shown(Sweep::Done)).unwrap();
    let calls = scene.host.calls();
    let removed = REMOVALS.map(|removal| calls.iter().position(|call| call.ends_with(removal)));
    let listed = calls.iter().filter(|call| call.contains(REMOTE_REFS)).count();
    assert!(
        removed[0].zip(removed[1]).is_some_and(|(remove, unbranch)| remove < unbranch)
            && listed == SWEEP_AND_RECHECK_LISTINGS,
        "{calls:?}"
    );
}

#[test]
fn a_failed_removal_keeps_the_worktree_and_no_candidate_means_no_scan() {
    let mut failing = Scene::new(&[PATH]);
    failing.host.fails("remove");
    let shown = format!("{}{}", failing.shown(Sweep::Done), Scene::new(&[LIVE]).shown(Sweep::Done));
    Golden::for_test(module_path!(), "kept").checked(&shown).unwrap();
}

#[test]
fn processes_that_could_not_be_inspected_are_counted_after_the_verdicts() {
    let one = Uninspected::default().counted();
    let shown = [one, one.counted()].map(|uninspected| {
        let mut scene = Scene::new(&[LIVE]);
        scene.host.uninspected_are(uninspected);
        scene.swept(Sweep::Planned).to_string()
    });
    assert!(shown[0].ends_with(UNINSPECTED[0]) && shown[1].ends_with(UNINSPECTED[1]), "{shown:?}");
}
