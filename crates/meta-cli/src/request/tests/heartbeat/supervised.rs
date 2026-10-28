use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::{EnvPolicy, Run};

use crate::request::heartbeat::Tick;
use crate::request::output::Output;
use crate::request::project::TickLimit;

const PATIENT: Duration = Duration::from_secs(10);
const IMPATIENT: Duration = Duration::from_millis(10);
const SHELL: &str = "sh";
const CLEAN: &str = "echo clean";
const RAISED: &str = "echo raised; exit 1";
const FAILED: &str = "echo oops >&2; exit 1";
const HANGS: &str = "sleep 30 & echo $! > \"$0\"; wait";
const ALIVE: [&str; 2] = ["kill", "-0"];
const POLL: Duration = Duration::from_millis(10);
const OOPS: &str = "oops";
const GAVE_UP: &str = "tick gave up after";
const TIMED_OUT: &str = "timed out";

struct Shell;

impl Shell {
    pub fn running(script: &str) -> Run {
        Run::new(SHELL, EnvPolicy::Inherit).args(["-c", script])
    }

    pub fn gone(pid: &Path) -> bool {
        let pid = std::fs::read_to_string(pid).unwrap_or_default();
        let alive =
            || Run::new(ALIVE[0], EnvPolicy::Inherit).args([ALIVE[1], pid.trim()]).stdout(Instant::now() + PATIENT);
        (0..PATIENT.as_millis() / POLL.as_millis()).any(|_| {
            std::thread::sleep(POLL);
            alive().is_err()
        })
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
fn an_inner_failure_and_a_run_past_the_limit_are_both_refused() {
    let failed = Tick::supervised(Shell::running(FAILED), TickLimit::from(PATIENT)).unwrap_err();
    let pid = tempfile::NamedTempFile::new().unwrap();
    let hung = Tick::supervised(Shell::running(HANGS).arg(pid.path()), TickLimit::from(IMPATIENT)).unwrap_err();
    assert!(failed.to_string().contains(OOPS) && hung.to_string().contains(GAVE_UP));
    assert!(hung.to_string().contains(TIMED_OUT) && Shell::gone(pid.path()));
}
