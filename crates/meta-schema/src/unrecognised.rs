use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected {expected}, found `{found}`")]
pub struct Unrecognised {
    expected: &'static str,
    found: Unparsed,
}

impl Unrecognised {
    #[must_use]
    pub fn new(expected: &'static str, found: &str) -> Self {
        Self { expected, found: Unparsed::from(found) }
    }
}
