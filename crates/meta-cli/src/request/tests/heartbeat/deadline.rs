use std::sync::mpsc;
use std::time::Duration;

use crate::request::heartbeat::Deadline;

const PATIENT: Duration = Duration::from_secs(5);
const IMPATIENT: Duration = Duration::from_millis(1);

#[test]
fn a_deadline_returns_finished_work() {
    assert_eq!(Deadline::within(PATIENT, || 7), Some(7));
}

#[test]
fn a_deadline_gives_up_on_work_that_never_finishes() {
    let (_held, blocked) = mpsc::channel::<()>();
    assert_eq!(Deadline::within(IMPATIENT, move || blocked.recv().is_ok()), None);
}
