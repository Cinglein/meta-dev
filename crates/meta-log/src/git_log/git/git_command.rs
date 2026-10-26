use std::ffi::OsStr;
use std::fmt;
use std::io::{self, Write as _};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use crate::detail::Detail;
use crate::git_log::content::Content;
use crate::git_log::sha::Sha;
use crate::log_failure::LogFailure;

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

    pub fn env(mut self, variable: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.command.env(variable, value);
        self
    }

    pub fn input(self, content: Content) -> Self {
        Self { input: Some(content), ..self }
    }

    pub fn timed_output(mut self) -> Result<Output, LogFailure> {
        let mut child = self.command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let deadline = Instant::now() + Self::NETWORK_TIMEOUT;
        let finished = std::iter::from_fn(|| deadline.checked_duration_since(Instant::now())).find_map(|_| {
            std::thread::sleep(Self::POLL);
            child.try_wait().transpose()
        });
        if let Some(status) = finished {
            status.and_then(|_| child.wait_with_output()).map_err(LogFailure::from)
        } else {
            child.kill()?;
            Err(LogFailure::Offline(Detail::from_reason(&format!(
                "{self} timed out after {:?}",
                Self::NETWORK_TIMEOUT
            ))))
        }
    }

    pub fn content(mut self) -> Result<Content, LogFailure> {
        let shown = self.to_string();
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
            Err(LogFailure::Git(Detail::from_reason(&format!("{shown}: {}", Detail::from(&output)))))
        }
    }

    pub fn sha(self) -> Result<Sha, LogFailure> {
        let content = self.content()?;
        Ok(content.as_ref().parse().unwrap_or_else(|never| match never {}))
    }
}

impl fmt::Display for GitCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let words: Vec<_> = self.command.get_args().map(OsStr::to_string_lossy).collect();
        write!(formatter, "git {}", words.join(" "))
    }
}
