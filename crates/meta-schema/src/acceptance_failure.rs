use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AcceptanceFailure {
    #[error("an acceptance needs at least one budget row and one span")]
    Empty,
    #[error("an acceptance needs at least one budget or step enforcement point")]
    Ungated,
}
