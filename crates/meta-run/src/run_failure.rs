use std::io;

use thiserror::Error;

use crate::command_line::CommandLine;

#[derive(Debug, Error)]
pub enum RunFailure {
    #[error("{line}: {source}")]
    Unspawned { line: CommandLine, source: io::Error },
    #[error("`{0}` timed out")]
    TimedOut(CommandLine),
    #[error("{complaint}")]
    Failed { line: CommandLine, complaint: String },
}
