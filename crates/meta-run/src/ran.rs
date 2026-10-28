use std::process::ExitStatus;

use crate::command_line::CommandLine;
use crate::run_failure::RunFailure;
use crate::stderr::Stderr;
use crate::stdout::Stdout;

#[derive(Debug)]
pub struct Ran {
    line: CommandLine,
    status: ExitStatus,
    stdout: Stdout,
    stderr: Stderr,
}

impl Ran {
    pub(crate) fn new(line: CommandLine, status: ExitStatus, stdout: Stdout, stderr: Stderr) -> Self {
        Self { line, status, stdout, stderr }
    }

    #[must_use]
    pub fn success(&self) -> bool {
        self.status.success()
    }

    #[must_use]
    pub fn stdout(&self) -> &Stdout {
        &self.stdout
    }

    #[must_use]
    pub fn stderr(&self) -> &Stderr {
        &self.stderr
    }

    #[must_use]
    pub fn line(&self) -> &CommandLine {
        &self.line
    }

    #[must_use]
    pub fn into_stdout(self) -> Stdout {
        self.stdout
    }

    pub fn succeeded(self) -> Result<String, RunFailure> {
        let complaint = self.stderr.first_line().unwrap_or_default().to_owned();
        let output = self.stdout.text().into_owned();
        self.status.success().then_some(output).ok_or(RunFailure::Failed { line: self.line, complaint })
    }
}
