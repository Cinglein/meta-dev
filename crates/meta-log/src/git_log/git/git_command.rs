use std::ffi::OsStr;
use std::fmt;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use crate::detail::Detail;
use crate::git_log::content::Content;
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
        let polls = Self::NETWORK_TIMEOUT.as_millis() / Self::POLL.as_millis();
        let finished = (0..polls).find_map(|_| {
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
            .map(|(content, mut pipe)| pipe.write_all(content.to_string().as_bytes()))
            .transpose()?;
        let output = child.wait_with_output()?;
        output
            .status
            .success()
            .then(|| Content::from(String::from_utf8_lossy(&output.stdout).trim_end().to_owned()))
            .ok_or_else(|| LogFailure::Git(Detail::from_reason(&format!("{shown}: {}", Detail::from(&output)))))
    }
}

impl fmt::Display for GitCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let words: Vec<_> = self.command.get_args().map(OsStr::to_string_lossy).collect();
        write!(formatter, "git {}", words.join(" "))
    }
}
