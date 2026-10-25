mod failure_kind;
mod invalid;
mod message;
mod misuse;
mod setup;
mod usage;

use std::fmt;

use meta_log::LogFailure;
use meta_state::Refusal;
use thiserror::Error;

use failure_kind::FailureKind;
use message::Message;
use usage::Usage;

pub use invalid::Invalid;
pub use misuse::Misuse;
pub use setup::Setup;

#[derive(Debug, Error)]
pub enum CliFailure {
    #[error("{0}\n{usage}", usage = Usage)]
    Usage(Message),
    #[error("{0}")]
    Invalid(Message),
    #[error("{0}")]
    Setup(Message),
    #[error("{0}")]
    Log(#[from] LogFailure),
    #[error("refused: {0}")]
    Refused(#[from] Refusal),
}

impl CliFailure {
    pub fn raised<Kind: FailureKind>(message: impl fmt::Display) -> Self {
        Kind::failure(Message::new(message))
    }
}
