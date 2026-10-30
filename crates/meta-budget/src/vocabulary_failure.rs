use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VocabularyFailure {
    #[error("a budget id is not empty")]
    EmptyBudget,
    #[error("a measure name is lowercase letters, digits and underscores, with a letter")]
    MisspelledMeasure,
    #[error("an amount is a count (0), a duration (250ms) or a share (30%)")]
    UnreadableAmount,
}
