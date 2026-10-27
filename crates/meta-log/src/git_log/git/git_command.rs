use std::ffi::OsStr;
use std::io::{self, Write as _};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use meta_trace::SpawnSpan;

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

    pub fn timed_output(mut self) -> Result<Output, LogFailure> {
        let span = SpawnSpan::opened(&self.command);
        let mut child = self.command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let deadline = Instant::now() + Self::NETWORK_TIMEOUT;
        let finished = std::iter::from_fn(|| deadline.checked_duration_since(Instant::now())).find_map(|_| {
            std::thread::sleep(Self::POLL);
            child.try_wait().transpose()
        });
        if let Some(status) = finished {
            let output = status.and_then(|_| child.wait_with_output());
            span.ended(output.as_ref().ok().map(|output| &output.status));
            output.map_err(LogFailure::from)
        } else {
            child.kill()?;
            span.ended(None);
            Err(LogFailure::TimedOut { command: CommandLine::from(&self.command), after: Self::NETWORK_TIMEOUT })
        }
    }

    pub fn content(mut self) -> Result<Content, LogFailure> {
        let command = CommandLine::from(&self.command);
        let span = SpawnSpan::opened(&self.command);
        let stdin = self.input.as_ref().map_or_else(Stdio::null, |_| Stdio::piped());
        let mut child = self.command.stdin(stdin).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        self.input
            .as_ref()
            .zip(child.stdin.take())
            .map(|(content, mut pipe)| pipe.write_all(content.as_ref().as_bytes()))
            .transpose()?;
        let output = child.wait_with_output()?;
        span.ended(Some(&output.status));
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
        Ok(content.as_ref().parse().unwrap_or_else(|never| match never {}))
    }
}
