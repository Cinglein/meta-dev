use thiserror::Error;

use crate::percent::Percent;

#[derive(Debug, Error)]
pub enum WarningFailure {
    #[error("warning line {0}: must be below the blocking line at 100")]
    AtTheBlockingLine(Percent),
}
