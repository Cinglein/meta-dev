use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TextFailure {
    #[error("text is empty")]
    Empty,
    #[error("text must be one line without control characters")]
    ControlCharacter,
    #[error("stored text has leading or trailing whitespace")]
    Untrimmed,
}
