use thiserror::Error;

use crate::unparsed::Unparsed;

#[derive(Debug, Error)]
pub enum TargetFailure {
    #[error("`{0}` is not a concern or issue id")]
    Unknown(Unparsed),
}
