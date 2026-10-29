use std::time::Duration;

use meta_golden::Golden;
use meta_run::{EnvPolicy, Run};

use crate::request::heartbeat::Tick;
use crate::request::project::TickLimit;

const PATIENT: Duration = Duration::from_secs(10);
const IMPATIENT: Duration = Duration::from_millis(10);
const SHELL: &str = "sh";
const CLEAN: &str = "echo clean";
const RAISED: &str = "echo raised; exit 1";
const FAILED: &str = "echo oops >&2; exit 1";
const HANGS: &str = "sleep 30 & wait";
const OOPS: &str = "oops";
const GAVE_UP: &str = "tick gave up after";
const TIMED_OUT: &str = "timed out";

struct Shell;

impl Shell {
    pub fn running(script: &str) -> Run {
        Run::new(SHELL, EnvPolicy::Inherit).args(["-c", script])
    }
}

#[test]
fn a_supervised_tick_passes_on_the_output_and_the_verdict_of_the_inner_run() {
    let passed = [CLEAN, RAISED].map(|script| Tick::supervised(Shell::running(script), TickLimit::from(PATIENT)));
    Golden::for_test(module_path!(), "passed").debugged(&passed.map(Result::unwrap)).unwrap();
}

#[test]
fn an_inner_failure_and_a_run_past_the_limit_are_both_refused() {
    let failed = Tick::supervised(Shell::running(FAILED), TickLimit::from(PATIENT)).unwrap_err();
    let hung = Tick::supervised(Shell::running(HANGS), TickLimit::from(IMPATIENT)).unwrap_err();
    assert!(failed.to_string().contains(OOPS) && hung.to_string().contains(GAVE_UP));
    assert!(hung.to_string().contains(TIMED_OUT));
}
