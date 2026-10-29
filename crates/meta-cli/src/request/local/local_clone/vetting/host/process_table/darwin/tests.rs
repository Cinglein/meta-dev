use std::process::{Child, Command};
use std::time::{Duration, Instant};

use libproc::bsd_info::BSDInfo;
use libproc::proc_pid::pidinfo;

use super::Darwin;

const EXIT: &str = "true";
const PATIENCE: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(10);

struct Zombie(Child);

impl Zombie {
    fn spawned() -> Self {
        let zombie = Self(Command::new(EXIT).spawn().unwrap());
        let pid = i32::try_from(zombie.0.id()).unwrap();
        let started = Instant::now();
        let dead = || pidinfo::<BSDInfo>(pid, Darwin::WHOLE).map_or(true, |info| info.pbi_status == libc::SZOMB);
        let waiting = std::iter::repeat_with(|| std::thread::sleep(POLL)).take_while(|()| started.elapsed() < PATIENCE);
        waiting.take_while(|()| !dead()).for_each(drop);
        zombie
    }
}

impl Drop for Zombie {
    fn drop(&mut self) {
        drop(self.0.wait());
    }
}

#[test]
fn an_own_child_that_exited_but_was_not_reaped_does_not_stop_the_scan() {
    let zombie = Zombie::spawned();
    let held = Darwin::held();
    drop(zombie);
    assert!(held.is_ok(), "{held:?}");
}
