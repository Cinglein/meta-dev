use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VocabularyFailure {
    #[error("a budget id is not empty")]
    EmptyBudget,
    #[error("a measure name is lowercase letters, digits and underscores, with a letter")]
    MisspelledMeasure,
}
