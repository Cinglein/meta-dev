use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use libproc::bsd_info::BSDInfo;
use libproc::proc_pid::pidinfo;

use super::Darwin;
use super::fd_vnode::FdVnode;
use super::probe_failure::ProbeFailure;

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

const FOUND: &str = "/bin/sleep";
const ENOENT: &str = "return code = 0, errno = 2, message = 'No such file or directory'";
const EIO: &str = "return code = 0, errno = 5, message = 'Input/output error'";
const DEAD: i32 = i32::MAX;
const TABLE: usize = 512;
const EPERM: &str = "return code = 0, errno = 1, message = 'Operation not permitted'";

#[test]
fn a_missing_program_file_leaves_no_program_path_and_the_scan_continues() {
    let rows = [Ok(FOUND.into()), Err(ENOENT.to_owned()), Err(EPERM.to_owned()), Err(EIO.to_owned())]
        .map(|found| ProbeFailure::program(found.map_err(ProbeFailure::from)));
    let shape = matches!(&rows, [Ok(Some(_)), Ok(None), Err(ProbeFailure::Refused), Err(ProbeFailure::Failed(_))]);
    assert!(shape && rows[0].as_ref().is_ok_and(|path| path.as_deref() == Some(Path::new(FOUND))), "{rows:?}");
}

#[test]
fn a_probe_failure_stops_the_scan_only_for_an_owned_live_process_and_not_for_a_refused_one() {
    let own = i32::try_from(std::process::id()).unwrap();
    let excused = |pid: i32, owned: &[i32], reported: &str| {
        let failure = ProbeFailure::from(reported.to_owned());
        failure.excused(owned.contains(&pid), || Darwin { pid }.alive()).is_ok()
    };
    let rows = [
        excused(own, &[own], ENOENT),
        excused(own, &[], ENOENT),
        excused(DEAD, &[DEAD], ENOENT),
        excused(own, &[own], EPERM),
    ];
    assert_eq!(rows, [false, true, true, true]);
}

#[test]
fn a_descriptor_that_is_not_open_has_no_path_and_does_not_fail_the_probe() {
    let own = i32::try_from(std::process::id()).unwrap();
    assert!(matches!(FdVnode::read(own, DEAD, TABLE), Ok(None)));
}

#[test]
fn a_missing_file_from_any_other_probe_stays_unexplained_so_an_owned_live_process_fails_the_scan() {
    assert!(ProbeFailure::from(ENOENT.to_owned()).unexplained().is_some());
}

#[test]
fn an_own_child_that_exited_but_was_not_reaped_does_not_stop_the_scan() {
    let zombie = Zombie::spawned();
    let held = Darwin::held();
    drop(zombie);
    assert!(held.is_ok(), "{held:?}");
}
