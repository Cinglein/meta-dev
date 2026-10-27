use thiserror::Error;

use crate::division::Division;
use crate::level_name::LevelName;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScaleFailure {
    #[error("scale {0}: no levels")]
    Empty(Division),
    #[error("scale {0}: {1} twice")]
    Twice(Division, LevelName),
    #[error("scale {0}: {1} maps onto a parent, but a top division has none")]
    TopMapped(Division, LevelName),
    #[error("scale {0}: {1} maps onto no level of the parent's scale")]
    Unmapped(Division, LevelName),
    #[error("scale {0}: {1} maps above a more severe level")]
    OrderBroken(Division, LevelName),
}
