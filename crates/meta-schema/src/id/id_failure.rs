use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdFailure {
    #[error("expected a {expected} id, found `{found}`")]
    WrongKind { expected: &'static str, found: Unparsed },
    #[error("an id ends in 16 lowercase hex digits")]
    Malformed,
}
