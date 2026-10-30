use std::fmt;

use meta_acceptance::Enforcement;
use meta_budget::BudgetId;
use serde::{Deserialize, Serialize};

use crate::gap::Gap;
use crate::verdict_visitor::VerdictVisitor;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Instrumented,
    Uninstrumented(Gap),
    OnTarget,
    OffTarget(BudgetId),
    NotGating(Enforcement),
}

impl Verdict {
    pub fn visit<Visitor: VerdictVisitor>(&self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::Instrumented => visitor.instrumented(),
            Self::Uninstrumented(_) => visitor.uninstrumented(),
            Self::OnTarget => visitor.on_target(),
            Self::OffTarget(_) | Self::NotGating(_) => visitor.failing(),
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instrumented => formatter.write_str("instrumented"),
            Self::Uninstrumented(gap) => write!(formatter, "uninstrumented {gap}"),
            Self::OnTarget => formatter.write_str("on target"),
            Self::OffTarget(budget) => write!(formatter, "off target {budget}"),
            Self::NotGating(point) => write!(formatter, "not gating {point}"),
        }
    }
}
