use std::io;

use meta_run::{CommandLine, RunFailure, Stderr};
use thiserror::Error;

use crate::git_log::{NotCommitId, Sha};
use crate::remote_name::RemoteName;
use crate::skipped_line::LineNumber;

#[derive(Debug, Error)]
pub enum LogFailure {
    #[error("offline: cannot fetch meta/log from {remote}: \"{stderr}\"; nothing written")]
    Offline { remote: RemoteName, stderr: Stderr },
    #[error("offline: {0}; nothing written")]
    Unrun(#[from] RunFailure),
    #[error("meta/log {0} is unreadable by this meta; upgrade meta before writing")]
    Unreadable(LineNumber),
    #[error("meta/log on {0} must hold only events.jsonl; nothing written")]
    Malformed(RemoteName),
    #[error("meta/log {0} is not in this clone; fetch it first")]
    Unfetched(Sha),
    #[error("git: {command}: {stderr}")]
    Git { command: CommandLine, stderr: Stderr },
    #[error("meta/log push rejected: {0}")]
    Rejected(Stderr),
    #[error("meta/log kept moving; gave up")]
    Contended,
    #[error("{0}")]
    NotCommit(#[from] NotCommitId),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Encode(#[from] serde_json::Error),
}
