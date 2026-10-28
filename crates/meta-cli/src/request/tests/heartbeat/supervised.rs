use std::time::Duration;

use meta_run::{EnvPolicy, KillScope, Run};

use crate::request::heartbeat::Tick;
use crate::request::output::Output;
use crate::request::project::TickLimit;
use crate::request::tests::pids::Pids;

const PATIENT: Duration = Duration::from_secs(10);
const IMPATIENT: Duration = Duration::from_millis(10);
const SHELL: &str = "sh";
const CLEAN: &str = "echo clean";
const RAISED: &str = "echo raised; exit 1";
const FAILED: &str = "echo oops >&2; exit 1";
const HANGS: &str = "sleep 30 & echo $! > \"$0\"; wait";
const OOPS: &str = "oops";
const GAVE_UP: &str = "tick gave up after";
const TIMED_OUT: &str = "timed out";

struct Shell;

impl Shell {
    pub fn running(script: &str) -> Run {
        Run::new(SHELL, EnvPolicy::Inherit).args(["-c", script]).scoped(KillScope::Group)
    }
}

#[test]
fn a_supervised_tick_passes_on_the_output_and_the_verdict_of_the_inner_run() {
    let clean = Tick::supervised(Shell::running(CLEAN), TickLimit::from(PATIENT)).unwrap();
    let raised = Tick::supervised(Shell::running(RAISED), TickLimit::from(PATIENT)).unwrap();
    assert!(matches!(&clean, Output::Listed(_)) && clean.to_string() == "clean\n");
    assert!(matches!(&raised, Output::Flagged(_)) && raised.to_string() == "raised\n");
}

#[test]
fn an_inner_failure_and_a_run_past_the_limit_are_both_refused_and_leave_no_process() {
    let failed = Tick::supervised(Shell::running(FAILED), TickLimit::from(PATIENT)).unwrap_err();
    let pid = tempfile::NamedTempFile::new().unwrap();
    let hung = Tick::supervised(Shell::running(HANGS).arg(pid.path()), TickLimit::from(IMPATIENT)).unwrap_err();
    assert!(failed.to_string().contains(OOPS) && hung.to_string().contains(GAVE_UP));
    assert!(hung.to_string().contains(TIMED_OUT) && Pids::gone(&pid, &[HANGS]));
}
