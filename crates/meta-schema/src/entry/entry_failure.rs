use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error)]
pub enum EntryFailure {
    #[error("`{0}` is not a concern, issue or task id")]
    Unknown(Unparsed),
}
