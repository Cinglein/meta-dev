use std::sync::mpsc;
use std::time::Duration;

use crate::request::project::TickLimit;

const PATIENT: Duration = Duration::from_secs(5);
const IMPATIENT: Duration = Duration::from_millis(1);
const ANSWER: &str = "done";

#[test]
fn a_deadline_returns_finished_work() {
    assert_eq!(TickLimit::from(PATIENT).within(|| ANSWER), Some(ANSWER));
}

#[test]
fn a_deadline_gives_up_on_work_that_never_finishes() {
    let (_held, blocked) = mpsc::channel::<()>();
    assert_eq!(TickLimit::from(IMPATIENT).within(move || blocked.recv().is_ok()), None);
}
