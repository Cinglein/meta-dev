use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use meta_run::Budget;

use super::deletable::Deletable;
use super::disk::Disk;
use super::scene::{Real, TASK};
use super::thaw::Thaw;
use super::world::World;
use crate::request::local::local_clone::quarantine::Quarantine;

const STAMP: &str = "1x0";
const WRITABLE: u32 = 0o200;
const PATIENT: Duration = Duration::from_secs(10);

#[test]
fn a_stranded_quarantine_is_restored_from_its_ledger_and_without_one_nothing_moves() {
    let real = Real::new(&[TASK]);
    let origin = real.world.trees().join(TASK);
    let quarantine = Quarantine::named(&origin, &STAMP.parse().unwrap());
    let common = real.world.common();
    let budget = Budget::new(PATIENT);
    let unledgered = Deletable::unstranded(&quarantine, &common, budget);
    real.world.git(&["worktree", "move", origin.to_str().unwrap(), quarantine.to_str().unwrap()]);
    World::run(&common, &["worktree", "lock", quarantine.to_str().unwrap()]);
    let plan = Thaw::planned(&origin, Thaw::ledger_of(&quarantine, &common));
    Disk::frozen(&quarantine, plan).unwrap();
    let restored = Deletable::unstranded(&quarantine, &common, budget);
    let mode = std::fs::metadata(origin.join("meta.toml")).unwrap().permissions().mode();
    assert!(unledgered.is_err() && restored.is_ok_and(|back| back == origin), "{origin:?}");
    assert!(!quarantine.exists() && mode & WRITABLE != 0 && !Thaw::ledger_of(&quarantine, &common).exists());
}
