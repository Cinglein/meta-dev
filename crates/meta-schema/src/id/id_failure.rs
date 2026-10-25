use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdFailure {
    #[error("expected a {expected} id, found `{found}`")]
    WrongKind { expected: &'static str, found: String },
    #[error("an id ends in 16 lowercase hex digits")]
    Malformed,
}
