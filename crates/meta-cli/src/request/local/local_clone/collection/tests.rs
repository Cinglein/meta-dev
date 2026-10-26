use std::path::PathBuf;

use chrono::Utc;
use meta_status::{Ending, Lifetime};

use super::{Census, Collection, Sweeper, Tracked};
use crate::request::local::local_clone::vetting::EntryKind;
use crate::request::local::local_clone::vetting::fake_host::{FakeHost, PATH, TASK};
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::sweep::Sweep;

const ESCAPED: &str = "/elsewhere/task-00000000000000b1";
const GONE: &str = "/trees/gone";
const LIVE: &str = "/trees/live";
const STRAY: &str = "/trees/stray";
const PLANNED: &str = "would free /trees/task-00000000000000b1: 1160 bytes; pr #1 merged
kept /elsewhere/task-00000000000000b1: path escapes the worktree root; pr #1 merged
would close /trees/gone: missing
kept /trees/live: pr #2 open
unrecorded /trees/stray: left alone
";
const DONE: [&str; 3] = [
    "freed /trees/task-00000000000000b1: 1160 bytes; pr #1 merged\n",
    "kept /elsewhere/task-00000000000000b1: path escapes the worktree root; pr #1 merged\n",
    "closed /trees/gone: missing\n",
];
const TRAILED: [&str; 3] = [
    r#""change":"Freed","path":"/trees/task-00000000000000b1""#,
    r#""change":"Refused","path":"/elsewhere/task-00000000000000b1""#,
    r#""change":"Closed","path":"/trees/gone""#,
];
const REASONS: [&str; 3] = [
    r#""reason":"pr #1 merged","bytes":1160}"#,
    r#""reason":"path escapes the worktree root; pr #1 merged"}"#,
    r#""at":"#,
];
const REMOVALS: [&str; 2] =
    ["worktree remove /trees/.gc-task-00000000000000b1", "update-ref -d refs/heads/task-00000000000000b1 c1"];
const UNREMOVED: &str = "kept /trees/task-00000000000000b1: removal failed: offline; pr #1 merged\n";
const LIVE_ONLY: &str = "kept /trees/live: pr #2 open\n";

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
        Self { host, census: Census::new(tracked, [PATH, STRAY].map(PathBuf::from).to_vec()) }
    }

    fn swept(&self, sweep: Sweep) -> Collection {
        let context = self.host.context();
        Collection::swept(&Sweeper::new(sweep, &context, &self.host), &self.census)
    }
}

#[test]
fn a_dry_run_vets_and_scans_once_but_deletes_and_trails_nothing() {
    let scene = Scene::new(&[PATH, ESCAPED, GONE, LIVE]);
    let planned = scene.swept(Sweep::Planned);
    assert_eq!(planned.to_string(), PLANNED);
    assert!(planned.trailed(Utc::now()).is_empty() && scene.host.scans() == [vec![PathBuf::from(PATH)]]);
    assert!(!scene.host.calls().iter().any(|call| REMOVALS.iter().any(|removal| call.ends_with(removal))));
}

#[test]
fn a_sweep_re_vets_removes_the_worktree_then_its_branch_and_trails_each_removal_refusal_and_closure() {
    let scene = Scene::new(&[PATH, ESCAPED, GONE, LIVE]);
    let done = scene.swept(Sweep::Done);
    let trailed: Vec<String> =
        done.trailed(Utc::now()).iter().map(|entry| serde_json::to_string(entry).unwrap()).collect();
    assert!(DONE.iter().all(|line| done.to_string().contains(line)), "{done}");
    assert!(
        trailed
            .iter()
            .zip(TRAILED)
            .zip(REASONS)
            .all(|((entry, change), reason)| { entry.starts_with(&["{", change].concat()) && entry.contains(reason) }),
        "{trailed:?}"
    );
    let calls = scene.host.calls();
    assert!(calls.iter().rev().take(2).rev().zip(REMOVALS).all(|(call, removal)| call.ends_with(removal)), "{calls:?}");
}

#[test]
fn a_failed_removal_keeps_the_worktree_and_no_candidate_means_no_scan() {
    let mut failing = Scene::new(&[PATH]);
    failing.host.fails("remove");
    assert!(failing.swept(Sweep::Done).to_string().starts_with(UNREMOVED));
    let live = Scene::new(&[LIVE]);
    assert!(live.swept(Sweep::Done).to_string().starts_with(LIVE_ONLY) && live.host.scans().is_empty());
}
