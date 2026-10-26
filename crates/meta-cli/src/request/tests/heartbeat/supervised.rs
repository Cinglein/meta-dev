use std::process::Command;
use std::time::Duration;

use crate::request::heartbeat::Tick;
use crate::request::output::Output;
use crate::request::project::TickLimit;

const PATIENT: Duration = Duration::from_secs(10);
const IMPATIENT: Duration = Duration::from_millis(100);
const SHELL: &str = "sh";
const CLEAN: &str = "echo clean";
const RAISED: &str = "echo raised; exit 1";
const FAILED: &str = "echo oops >&2; exit 1";
const HANGS: &str = "sleep 30 & wait";

struct Shell;

impl Shell {
    pub fn running(script: &str) -> Command {
        let mut command = Command::new(SHELL);
        command.args(["-c", script]);
        command
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
    let failed = Tick::supervised(Shell::running(FAILED), TickLimit::from(PATIENT)).unwrap_err().to_string();
    let hung = Tick::supervised(Shell::running(HANGS), TickLimit::from(IMPATIENT)).unwrap_err().to_string();
    assert!(failed.contains("oops") && hung.contains("tick gave up after") && hung.contains("timed out"));
}
