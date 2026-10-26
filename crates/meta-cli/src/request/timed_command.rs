mod drain;
mod process_group;

use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::request::output::Listed;
use drain::Drain;
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

    pub fn stdout(mut self) -> Result<String, Listed> {
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
        let succeeded = child.wait().map_err(Listed::new)?.success();
        let late = (Instant::now() >= self.deadline).then(|| Listed::new(Self::TIMED_OUT));
        let failed = || late.unwrap_or_else(|| Listed::new(errors.lines().next().unwrap_or_default()));
        succeeded.then_some(output).ok_or_else(failed)
    }
}
