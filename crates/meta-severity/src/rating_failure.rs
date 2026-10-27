use meta_schema::LevelName;
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
}
