use meta_budget::BudgetId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AcceptanceFailure {
    #[error("an acceptance needs at least one budget or step enforcement point")]
    Ungated,
    #[error("budget {0} has two acceptance rows")]
    DuplicateBudget(BudgetId),
    #[error("the list needs at least one entry")]
    Empty,
    #[error("a row is <budget>:<measure><=<target>, for example speed:test_time<=0")]
    UnreadableRow,
    #[error("an enforcement point is budget:<id>, step:<name> or hook:<name>")]
    UnreadablePoint,
}
