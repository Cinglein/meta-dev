use std::io;

use thiserror::Error;

use crate::command_line::CommandLine;

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
    Failed { line: CommandLine, complaint: String },
}
