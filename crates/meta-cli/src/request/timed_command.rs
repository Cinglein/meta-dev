mod drain;
mod finished;
mod process_group;

use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::request::output::Listed;
use drain::Drain;
pub use finished::Finished;
use process_group::ProcessGroup;

#[derive(Debug)]
pub struct TimedCommand {
    command: Command,
    deadline: Instant,
}

impl TimedCommand {
    const TIMED_OUT: &'static str = "timed out";

    pub fn new(command: Command, deadline: Instant) -> Self {
        Self { command, deadline }
    }

    pub fn stdout(self) -> Result<String, Listed> {
        self.finished()?.succeeded()
    }

    pub fn finished(mut self) -> Result<Finished, Listed> {
        let mut child = self
            .command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(i32::default())
            .spawn()
            .map_err(Listed::new)?;
        let group = ProcessGroup::led_by(&child);
        let output = Drain::reading(child.stdout.take());
        let errors = Drain::reading(child.stderr.take());
        let output = output.within(self.deadline, group);
        let errors = errors.within(self.deadline, group);
        let status = child.wait().map_err(Listed::new)?;
        (status.success() || self.deadline.checked_duration_since(Instant::now()).is_some())
            .then(|| Finished::new(status, output, errors))
            .ok_or_else(|| Listed::new(Self::TIMED_OUT))
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use super::TimedCommand;

    const HELD: [&str; 2] = ["-c", "set -m; sleep 2 & echo held"];
    const LIMIT: Duration = Duration::from_millis(100);
    const PROMPT: Duration = Duration::from_secs(1);

    #[test]
    fn a_grandchild_in_another_group_holding_the_pipes_cannot_outlast_the_deadline() {
        let mut command = Command::new("bash");
        command.args(HELD);
        let started = Instant::now();
        let finished = TimedCommand::new(command, started + LIMIT).finished();
        assert!(finished.is_err() && started.elapsed() < PROMPT);
    }
}
