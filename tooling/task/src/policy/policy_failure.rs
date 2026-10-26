use thiserror::Error;

use super::budgets::{LegacyKey, TreeFailure};

#[derive(Debug, Error)]
pub enum PolicyFailure {
    #[error(transparent)]
    Tree(#[from] TreeFailure),
    #[error("{0}: also set as a budget; drop the old key")]
    Twice(LegacyKey),
    #[error("{0}: not set; declare it as a budget")]
    Unset(LegacyKey),
    #[error("[{0}]: a key is not set")]
    Missing(&'static str),
}
