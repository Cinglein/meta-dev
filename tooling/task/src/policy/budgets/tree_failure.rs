use thiserror::Error;

use super::budget_id::BudgetId;
use crate::policy::domains::ScopeKind;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeFailure {
    #[error("budget {0}: part of a budget that is not defined")]
    NoParent(BudgetId),
    #[error("budget {0}: its division is not inside the division of the budget it is part of")]
    NotNested(BudgetId),
    #[error("budget {0}: its parts add up to more than all of it")]
    Oversubscribed(BudgetId),
    #[error("budget {0}: a budget in a nested division is a part of its parent division's budget")]
    NestedWhole(BudgetId),
    #[error("budget {0}: a part has the window of the budget it is part of")]
    OtherWindow(BudgetId),
    #[error("budget {0}: its allowance is not in its measure's unit")]
    OtherUnit(BudgetId),
    #[error("budget {0}: its measure needs {1}, which its division's domain does not declare")]
    Unmeasurable(BudgetId, ScopeKind),
}
