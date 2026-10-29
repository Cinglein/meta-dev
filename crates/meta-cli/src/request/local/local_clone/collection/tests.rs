use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use meta_golden::Golden;
use meta_status::{Ending, Lifetime};

use super::{Census, Collection, Sweeper, Tracked};
use crate::request::local::local_clone::quarantine::Stamp;
use crate::request::local::local_clone::vetting::scene::Scene;
use crate::request::local::local_clone::vetting::{Host, Interpreter};
use crate::request::local::sweep::Sweep;

const PATH: &str = Scene::PATH;
const ESCAPED: &str = "/elsewhere/task-00000000000000b1";
const GONE: &str = "/trees/gone";
const LIVE: &str = "/trees/live";
const STRAY: &str = "/trees/stray";
const LOST: &str = "/trees/lost";
const KEPT_QUARANTINE: &str = "/trees/.gc-kept-2x0";
const LISTED: [&str; 4] = [PATH, STRAY, "/trees/.gc-lost-1x0", "/trees/.gc-old-9x9"];
const SWEPT: [&str; 6] = [PATH, ESCAPED, GONE, LIVE, LOST, KEPT_QUARANTINE];
const AT: &str = "2026-10-27T12:00:00Z";
const STAMP: &str = "1x0";
const QUARANTINE: &str = "/trees/.gc-task-00000000000000b1-1x0";
const LOCKED_DIR: &str = "target/debug";
const READ_ONLY: u32 = 0o555;
const WRITABLE: u32 = 0o755;

struct Bench {
    scene: Scene,
    census: Census,
}

impl Bench {
    fn new(tracked: &[&str]) -> Self {
        let scene = Scene::new();
        [ESCAPED, LIVE].iter().for_each(|place| std::fs::create_dir_all(scene.real(place)).unwrap());
        let lifetime = |path: &str| match path {
            LIVE => Lifetime::Open("2".parse().unwrap()),
            _ => Lifetime::Ended(Ending::Merged("1".parse().unwrap())),
        };
        let tracked = tracked.iter().map(|place| Tracked::new(scene.recorded(place), lifetime(place))).collect();
        let census = Census::new(tracked, LISTED.map(|place| scene.real(place)).to_vec());
        Self { scene, census }
    }

    fn shown(&self, sweep: Sweep) -> String {
        let host = Host::default();
        let interpreter = Interpreter::new(&host);
        let Ok(stamp) = STAMP.parse::<Stamp>();
        let sweeper = Sweeper::new(sweep, self.scene.context(), &interpreter, stamp);
        let surveyed = sweeper.surveyed(&self.census).unwrap();
        let collection = Collection::swept(&sweeper.lent(&surveyed), &self.census, &surveyed);
        let trailed = collection.trailed(AT.parse::<DateTime<Utc>>().unwrap());
        let trailed: Vec<String> = trailed.iter().map(|entry| serde_json::to_string(entry).unwrap()).collect();
        self.scene.shown(&format!("{collection}{trailed:#?}\n"))
    }

    fn branched(&self) -> bool {
        !self.scene.git(&self.scene.real("/clone"), &["branch", "--list", Scene::TASK]).is_empty()
    }

    fn sized(&self) -> String {
        let path = self.scene.real(PATH);
        let sizes = [".git", "meta.toml", "target/CACHEDIR.TAG"].map(|name| std::fs::metadata(path.join(name)).unwrap().len());
        sizes.iter().sum::<u64>().to_string()
    }
}

#[test]
fn a_dry_run_vets_but_deletes_and_trails_nothing() {
    let bench = Bench::new(&SWEPT);
    let shown = bench.shown(Sweep::Planned).replace(&bench.sized(), "N");
    Golden::for_test(module_path!(), "planned").checked(&shown).unwrap();
    assert!(bench.scene.real(PATH).exists() && bench.branched());
}

#[test]
fn a_sweep_re_vets_removes_the_worktree_then_its_branch_and_trails_each_removal_refusal_and_closure() {
    let bench = Bench::new(&SWEPT);
    let sized = bench.sized();
    let shown = bench.shown(Sweep::Done).replace(&sized, "N");
    Golden::for_test(module_path!(), "done").checked(&shown).unwrap();
    assert!(!bench.scene.real(PATH).exists() && !bench.branched());
}

#[test]
fn a_failed_removal_keeps_the_worktree_and_no_candidate_means_no_vet() {
    let failing = Bench::new(&[PATH]);
    let locked = failing.scene.real(PATH).join(LOCKED_DIR);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(READ_ONLY)).unwrap();
    let kept = failing.shown(Sweep::Done);
    // A real finding: git's own remove fails half way, and the move back then fails too, so the
    // remains stay in quarantine (reported as removal failed, branch kept). FakeHost could not show it.
    let stranded = failing.scene.real(QUARANTINE).join(LOCKED_DIR);
    assert!(stranded.exists() && failing.branched(), "{kept}");
    std::fs::set_permissions(&stranded, std::fs::Permissions::from_mode(WRITABLE)).unwrap();
    let _ = locked;
    let shown = format!("{kept}{}", Bench::new(&[LIVE]).shown(Sweep::Done));
    Golden::for_test(module_path!(), "kept").checked(&shown).unwrap();
    let _unused: Option<PathBuf> = None;
}
