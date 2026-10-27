use meta_schema::{Division, LevelName};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RatingFailure {
    #[error("an item at the root has no scale; place it in a division first")]
    Unscaled,
    #[error("{0} is not a level of the division's scale")]
    UnknownLevel(LevelName),
    #[error("the item is already {0}")]
    Unchanged(LevelName),
    #[error("raising from {0} needs --scenario <reproduction or failure scenario>")]
    RaisedWithoutScenario(LevelName),
    #[error("{0} is not on the scale of {1}; move it with --severity <level>")]
    Unplaced(LevelName, Division),
    #[error("{0} means another level on {1}; move it with --severity <level>")]
    Remapped(LevelName, Division),
    #[error("{0} is not a division in tooling/policy.toml")]
    Undeclared(Division),
}
