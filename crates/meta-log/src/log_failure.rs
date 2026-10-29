use std::time::Instant;

use meta_git::{GitFailure, RemoteName};
use meta_run::Stderr;
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
    #[error("meta/log fetch ran past its git_millis deadline; gave up")]
    Expired,
    #[error("{0}")]
    NotCommit(#[from] NotCommitId),
    #[error("{0}")]
    Encode(#[from] serde_json::Error),
}

impl LogFailure {
    #[must_use]
    pub fn gave_up(deadline: Instant) -> LogFailure {
        if Instant::now() < deadline { LogFailure::Contended } else { LogFailure::Expired }
    }
}

#[cfg(test)]
mod tests {
    use super::LogFailure;
    use std::time::{Duration, Instant};
    const AHEAD: Duration = Duration::from_secs(60);

    #[test]
    fn giving_up_before_the_deadline_is_contention_and_after_it_is_expiry() {
        let now = Instant::now();
        let given = [now + AHEAD, now].map(|deadline| matches!(LogFailure::gave_up(deadline), LogFailure::Expired));
        assert_eq!(given, [false, true]);
    }
}
