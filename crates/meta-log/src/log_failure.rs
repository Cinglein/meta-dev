use meta_git::{GitFailure, RemoteName};
use meta_run::Stderr;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

use crate::git_log::{NotCommitId, Sha};
use crate::skipped_line::LineNumber;

#[derive(Debug, Error)]
pub enum LogFailure {
    #[error("offline: cannot fetch meta/log from {remote}: \"{stderr}\"; nothing written")]
    Offline { remote: RemoteName, stderr: Stderr },
    #[error("{0}; nothing written")]
    Port(#[from] GitFailure),
    #[error("meta/log {0} is unreadable by this meta; upgrade meta before writing")]
    Unreadable(LineNumber),
    #[error("meta/log on {0} must hold only events.jsonl; nothing written")]
    Malformed(RemoteName),
    #[error("meta/log {0} is not in this clone; fetch it first")]
    Unfetched(Sha),
    #[error("meta/log push rejected: {0}")]
    Rejected(Stderr),
    #[error("meta/log kept moving; gave up")]
    Contended,
    #[error("meta/log kept moving; gave up; {} is {}s old; delete it if no meta process is running", .lock.display(), .age.as_secs())]
    StaleLock { lock: PathBuf, age: Duration },
    #[error("{0}")]
    NotCommit(#[from] NotCommitId),
    #[error("{0}")]
    Encode(#[from] serde_json::Error),
}
