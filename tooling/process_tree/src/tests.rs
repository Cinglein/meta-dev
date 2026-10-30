use std::process::{Child, Command};
use std::time::{Duration, Instant};

use super::{Host, ProcessTree, Victim};

const SLEEP: &str = "sleep";
const SECONDS: &str = "30";
const ABSENT: &str = "no-such-process-runs-here";
const LATER: Duration = Duration::from_secs(3600);

struct Sleeper(Child);

impl Sleeper {
    fn started() -> Sleeper {
        Sleeper(Command::new(SLEEP).arg(SECONDS).spawn().unwrap())
    }

    fn ended_by(mut self, killed: &[u32]) -> bool {
        killed == [self.0.id()] && !self.0.wait().unwrap().success()
    }
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

#[test]
fn a_tree_rooted_at_a_child_uses_memory_and_killing_its_largest_member_ends_the_child() {
    let child = Sleeper::started();
    let mut tree = ProcessTree::rooted_at(&child.0);
    let started = Instant::now();
    let kept = tree.sampled(started, LATER);
    let found = tree.sampled(started, Duration::ZERO);
    let killed: Vec<u32> = tree.largest_killed().iter().map(|victim| victim.pid().as_u32()).collect();
    assert!(kept == started && found > started && tree.used().count() > 0);
    assert!(child.ended_by(&killed));
}

#[test]
fn every_member_of_a_tree_is_killed_and_the_host_reports_free_memory_and_no_absent_process() {
    let child = Sleeper::started();
    let mut tree = ProcessTree::rooted_at(&child.0);
    tree.sampled(Instant::now(), Duration::ZERO);
    let killed: Vec<u32> = tree.all_killed().iter().map(|victim| victim.pid().as_u32()).collect();
    assert!(child.ended_by(&killed));
    let own = Victim::current().unwrap();
    let name = own.name().to_string_lossy().into_owned();
    assert!(Host::available().count() > 0 && !Host::runs(ABSENT) && Host::runs(&name) && own.bytes().count() > 0);
}
