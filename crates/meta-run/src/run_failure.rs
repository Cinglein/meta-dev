use std::io;
use std::process::ExitStatus;

use thiserror::Error;

use crate::command_line::CommandLine;
use crate::stderr::Stderr;
use crate::unread::Unread;

#[derive(Debug, Error)]
pub enum RunFailure {
    #[error("{line}: {source}")]
    Unspawned { line: CommandLine, source: io::Error },
    #[error("`{0}` timed out; a run waiting for terminal input stalls until its deadline")]
    TimedOut(CommandLine),
    #[error("`{0}` not started: its deadline had passed")]
    Expired(CommandLine),
    #[error("`{0}` not started: meta is shutting down")]
    Interrupted(CommandLine),
    #[error("{complaint}")]
    Failed { line: CommandLine, status: ExitStatus, complaint: String },
}

impl From<RunFailure> for Unread {
    fn from(failure: RunFailure) -> Self {
        match failure {
            RunFailure::Failed { status, complaint, .. } => Self::Failed(status, Stderr::said(complaint)),
            RunFailure::Unspawned { line, source } => Self::Unspawned(line, source.kind()),
            RunFailure::TimedOut(line) | RunFailure::Expired(line) | RunFailure::Interrupted(line) => Self::Late(line),
        }
    }
}
