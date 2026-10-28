mod pipe_bytes;
mod process_group;

use std::ffi::OsStr;
use std::io::{self, Read, Write as _};
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Output, Stdio};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use pipe_bytes::PipeBytes;
use process_group::ProcessGroup;

use crate::command_line::CommandLine;
use crate::git_log::content::Content;
use crate::git_log::sha::Sha;
use crate::log_failure::LogFailure;
use crate::stderr::Stderr;

#[derive(Debug)]
pub struct GitCommand {
    command: Command,
    input: Option<Content>,
}

impl GitCommand {
    const NETWORK_TIMEOUT: Duration = Duration::from_secs(30);
    const POLL: Duration = Duration::from_millis(1);

    pub fn new(command: Command) -> Self {
        Self { command, input: None }
    }

    pub fn arg(mut self, word: impl AsRef<OsStr>) -> Self {
        self.command.arg(word);
        self
    }

    pub fn input(self, content: Content) -> Self {
        Self { input: Some(content), ..self }
    }

    fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<io::Result<PipeBytes>> {
        std::thread::spawn(move || PipeBytes::read_all(pipe))
    }

    fn drained(handle: JoinHandle<io::Result<PipeBytes>>) -> io::Result<PipeBytes> {
        handle.join().unwrap_or_else(|_| Err(io::Error::other("output reader panicked")))
    }

    pub fn network_deadline() -> Instant {
        Instant::now() + Self::NETWORK_TIMEOUT
    }

    pub fn timed_output(mut self, deadline: Instant) -> Result<Output, LogFailure> {
        let started = Instant::now();
        let mut child = self
            .command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(i32::default())
            .spawn()?;
        let stdout = Self::drain(child.stdout.take());
        let stderr = Self::drain(child.stderr.take());
        let finished = std::iter::from_fn(|| deadline.checked_duration_since(Instant::now())).find_map(|_| {
            std::thread::sleep(Self::POLL);
            child.try_wait().transpose()
        });
        if let Some(status) = finished {
            let status = status?;
            Ok(Self::drained(stdout)?.into_output(status, Self::drained(stderr)?))
        } else {
            ProcessGroup::led_by(&child).kill()?;
            child.wait()?;
            let after = deadline.saturating_duration_since(started);
            Err(LogFailure::TimedOut { command: CommandLine::from(&self.command), after })
        }
    }

    pub fn content(mut self) -> Result<Content, LogFailure> {
        let command = CommandLine::from(&self.command);
        let stdin = self.input.as_ref().map_or_else(Stdio::null, |_| Stdio::piped());
        let mut child = self.command.stdin(stdin).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        self.input
            .as_ref()
            .zip(child.stdin.take())
            .map(|(content, mut pipe)| pipe.write_all(content.as_ref().as_bytes()))
            .transpose()?;
        let output = child.wait_with_output()?;
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map(Content::from)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error).into())
        } else {
            Err(LogFailure::Git { command, stderr: Stderr::from(&output) })
        }
    }

    pub fn sha(self) -> Result<Sha, LogFailure> {
        let content = self.content()?;
        Ok(content.as_ref().parse()?)
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::GitCommand;

    const PAST_PIPE_BUFFER: usize = 1 << 17;

    #[test]
    fn output_past_the_pipe_buffer_is_returned() {
        let mut command = Command::new("head");
        command.args(["-c", &PAST_PIPE_BUFFER.to_string(), "/dev/zero"]);
        let output =
            GitCommand::new(command).timed_output(GitCommand::network_deadline()).map(|output| output.stdout.len());
        assert_eq!(output.ok(), Some(PAST_PIPE_BUFFER));
    }
}
