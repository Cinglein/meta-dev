use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error)]
pub enum TagFailure {
    #[error("unknown tag `{0}`; known: acceptance-candidate")]
    Unknown(Unparsed),
}
