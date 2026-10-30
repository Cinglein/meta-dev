use meta_budget::BudgetId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AcceptanceFailure {
    #[error("an acceptance needs at least one budget or step enforcement point")]
    Ungated,
    #[error("budget {0} has two acceptance rows")]
    DuplicateBudget(BudgetId),
    #[error("a span, step or hook name is one trimmed line")]
    Blank,
}
