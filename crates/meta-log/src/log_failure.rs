use std::io;

use thiserror::Error;

use crate::detail::Detail;

#[derive(Debug, Error)]
pub enum LogFailure {
    #[error("refused: {0}")]
    Refused(Detail),
    #[error("offline: cannot fetch meta/log from {0}; nothing written")]
    Offline(Detail),
    #[error("meta/log has {0} line(s) this meta cannot read; upgrade meta before writing")]
    Unreadable(Detail),
    #[error("git: {0}")]
    Git(Detail),
    #[error("meta/log push rejected: {0}")]
    Rejected(Detail),
    #[error("meta/log kept moving; gave up after {0} attempts")]
    Contended(Detail),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Encode(#[from] serde_json::Error),
}

impl LogFailure {
    #[expect(clippy::needless_pass_by_value, reason = "map_err hands the refusal over by value")]
    pub fn refused(reason: impl ToString) -> Self {
        Self::Refused(Detail::from_reason(&reason))
    }
}
