mod enforcer;
mod item_filter;
mod item_kind;
mod mutation_job;
mod tally;
mod unit;

use serde::{Deserialize, Serialize};
use strum::Display;

use super::window::Window;
use crate::domains::ScopeKind;
pub use enforcer::Enforcer;
pub use item_filter::ItemFilter;
pub use item_kind::ItemKind;
pub use mutation_job::MutationJob;
pub use tally::Tally;
pub use unit::Unit;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Measure {
    TestWallTime,
    SlowestTest,
    #[strum(to_string = "workflow_wall_time {0}")]
    WorkflowWallTime(MutationJob),
    MutantPrice,
    LintFindings,
    TestLineShare,
    TangleItems,
    #[strum(to_string = "open_items {0}")]
    OpenItems(ItemFilter),
    Words,
}

impl Measure {
    #[must_use]
    pub const fn unit(&self) -> Unit {
        match self {
            Measure::TestWallTime | Measure::SlowestTest | Measure::WorkflowWallTime(_) => {
                Unit::Duration
            }
            Measure::MutantPrice | Measure::TestLineShare => Unit::Share,
            Measure::TangleItems | Measure::OpenItems(_) | Measure::Words => Unit::Count,
            Measure::LintFindings => Unit::Baseline,
        }
    }

    #[must_use]
    pub const fn scope(&self) -> Option<ScopeKind> {
        match self {
            Measure::TestWallTime | Measure::SlowestTest => Some(ScopeKind::Tests),
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => Some(ScopeKind::Workflows),
            Measure::TangleItems => Some(ScopeKind::Crates),
            Measure::LintFindings | Measure::TestLineShare | Measure::Words => {
                Some(ScopeKind::Paths)
            }
            Measure::OpenItems(_) => None,
        }
    }

    #[must_use]
    pub const fn enforcer(&self) -> Enforcer {
        match self {
            Measure::TestWallTime
            | Measure::SlowestTest
            | Measure::OpenItems(_)
            | Measure::Words => Enforcer::Checker,
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => Enforcer::MutationJobs,
            Measure::LintFindings | Measure::TestLineShare | Measure::TangleItems => Enforcer::Lint,
        }
    }

    #[must_use]
    pub const fn item_filter(&self) -> Option<&ItemFilter> {
        match self {
            Measure::OpenItems(filter) => Some(filter),
            Measure::TestWallTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems
            | Measure::Words => None,
        }
    }

    #[must_use]
    pub const fn tally(&self) -> Tally {
        match self {
            Measure::Words => Tally::DomainFiles,
            Measure::TestWallTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems
            | Measure::OpenItems(_) => Tally::WholeScope,
        }
    }

    #[must_use]
    pub const fn window(&self) -> Window {
        match self {
            Measure::OpenItems(_) | Measure::Words => Window::Standing,
            Measure::TestWallTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems => Window::PerChange,
        }
    }
}
