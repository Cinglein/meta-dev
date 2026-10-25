mod message;
mod usage;

use std::fmt;

use meta_log::LogFailure;
use meta_state::Refusal;
use thiserror::Error;

use message::Message;
use usage::Usage;

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
    pub fn usage(message: impl fmt::Display) -> Self {
        Self::Usage(Message::new(message))
    }

    pub fn invalid(message: impl fmt::Display) -> Self {
        Self::Invalid(Message::new(message))
    }

    pub fn config(message: impl fmt::Display) -> Self {
        Self::Setup(Message::new(message))
    }
}
