use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LevelNameFailure {
    #[error("severity level {0}: lowercase letters, digits and hyphens, with a letter")]
    Misspelled(Unparsed),
}
