use thiserror::Error;

use super::budget_id::BudgetId;
use crate::domains::ScopeKind;

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
    #[error("budget {0}: its allowance is not in its measure's unit")]
    OtherUnit(BudgetId),
    #[error("budget {0}: this id is kept for an old policy key")]
    Taken(BudgetId),
    #[error("budget {0}: its measure needs {1}, which its division's domain does not declare")]
    Unmeasurable(BudgetId, ScopeKind),
    #[error(
        "budget {0}: its division's domain covers only part of {1}; partial domains are not measured yet"
    )]
    PartlyMeasurable(BudgetId, ScopeKind),
    #[error("budget {0}: another top-level budget has its measure")]
    Duplicate(BudgetId),
    #[error("budget {0}: a lint enforces its measure; it takes no tolerance")]
    LintTolerance(BudgetId),
    #[error("budget {0}: its blocking line overflows")]
    Overflow(BudgetId),
    #[error("budget {0}: a baseline budget has no parts")]
    BaselinePart(BudgetId),
    #[error("budget {0}: open items take a standing window; every other measure, per change")]
    OtherWindow(BudgetId),
    #[error("budget {0}: its measure is not the measure of the budget it is part of")]
    OtherMeasure(BudgetId),
    #[error("budget {0}: its parts are split by both percent and count")]
    MixedParts(BudgetId),
}
