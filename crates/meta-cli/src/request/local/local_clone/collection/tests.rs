use chrono::{DateTime, Utc};
use meta_status::{Ending, Lifetime};

use super::{Census, Collection, Sweeper, Tracked};
use crate::request::local::local_clone::vetting::scene::Real;
use crate::request::local::sweep::Sweep;

const MERGED: &str = "task-00000000000000b1";
const LIVE: &str = "task-00000000000000b2";
const GONE: &str = "task-00000000000000b3";
const AT: &str = "2026-10-27T12:00:00Z";
const PLANNED: [&str; 3] = ["would free ", "kept ", "would close "];
const DONE: [&str; 3] = ["freed ", "kept ", "closed "];
const TRAILED: [&str; 2] = ["\"change\":\"Freed\"", "\"change\":\"Closed\""];

struct Scene(Real);

impl Scene {
    fn new() -> Self {
        Self(Real::new(&[MERGED, LIVE]))
    }

    fn shown(&self, sweep: Sweep) -> String {
        let context = self.0.context();
        let lifetime = |task: &str| match task {
            LIVE => Lifetime::Open("2".parse().unwrap()),
            _ => Lifetime::Ended(Ending::Merged("1".parse().unwrap())),
        };
        let tracked = [MERGED, LIVE, GONE].map(|task| Tracked::new(self.0.worktree(task), lifetime(task)));
        let listed = context.listing().as_ref().unwrap().paths(&self.0.world.checkout());
        let census = Census::new(tracked.to_vec(), listed);
        let collection = Collection::swept(&Sweeper::new(sweep, &context), &census);
        let trailed = collection.trailed(AT.parse::<DateTime<Utc>>().unwrap());
        let trailed: Vec<String> = trailed.iter().map(|entry| serde_json::to_string(entry).unwrap()).collect();
        format!("{collection}[{}]", trailed.join(","))
    }

    fn lines(&self, shown: &str, verbs: [&str; 3]) -> bool {
        let trees = self.0.world.trees();
        [MERGED, LIVE, GONE].iter().zip(verbs).all(|(task, verb)| {
            shown.lines().any(|line| line.starts_with(&format!("{verb}{}", trees.join(task).display())))
        })
    }
}

#[test]
fn a_dry_run_vets_but_deletes_and_trails_nothing() {
    let scene = Scene::new();
    let shown = scene.shown(Sweep::Planned);
    let trees = scene.0.world.trees();
    assert!(scene.lines(&shown, PLANNED) && shown.ends_with("[]"), "{shown}");
    assert!(trees.join(MERGED).exists() && scene.0.branch(MERGED), "{shown}");
}

#[test]
fn a_sweep_re_vets_removes_the_worktree_then_its_branch_and_trails_each_removal_and_closure() {
    let scene = Scene::new();
    let shown = scene.shown(Sweep::Done);
    let trees = scene.0.world.trees();
    assert!(scene.lines(&shown, DONE) && TRAILED.iter().all(|entry| shown.contains(entry)), "{shown}");
    assert!(!trees.join(MERGED).exists() && !scene.0.branch(MERGED), "{shown}");
    assert!(trees.join(LIVE).exists() && scene.0.branch(LIVE), "{shown}");
}
