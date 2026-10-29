mod enforcer;
mod item_filter;
mod item_kind;
mod mutation_job;
mod unit;

use serde::{Deserialize, Serialize};
use strum::Display;

use super::window::Window;
use crate::domains::ScopeKind;
pub use enforcer::Enforcer;
pub use item_filter::ItemFilter;
pub use item_kind::ItemKind;
pub use mutation_job::MutationJob;
pub use unit::Unit;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Measure {
    TestWallTime,
    TestCpuTime,
    SlowestTest,
    GitProcesses,
    #[strum(to_string = "workflow_wall_time {0}")]
    WorkflowWallTime(MutationJob),
    MutantPrice,
    LintFindings,
    TestLineShare,
    TangleItems,
    GitStarters,
    #[strum(to_string = "open_items {0}")]
    OpenItems(ItemFilter),
}

impl Measure {
    #[must_use]
    pub const fn unit(&self) -> Unit {
        match self {
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_) => Unit::Duration,
            Measure::MutantPrice | Measure::TestLineShare => Unit::Share,
            Measure::TangleItems
            | Measure::OpenItems(_)
            | Measure::GitProcesses
            | Measure::GitStarters => Unit::Count,
            Measure::LintFindings => Unit::Baseline,
        }
    }

    #[must_use]
    pub const fn scope(&self) -> Option<ScopeKind> {
        match self {
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::GitProcesses => Some(ScopeKind::Tests),
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => Some(ScopeKind::Workflows),
            Measure::TangleItems => Some(ScopeKind::Crates),
            Measure::LintFindings | Measure::TestLineShare => Some(ScopeKind::Paths),
            Measure::OpenItems(_) | Measure::GitStarters => None,
        }
    }

    #[must_use]
    pub const fn enforcer(&self) -> Enforcer {
        match self {
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::GitProcesses
            | Measure::GitStarters
            | Measure::OpenItems(_) => Enforcer::Checker,
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => Enforcer::MutationJobs,
            Measure::LintFindings | Measure::TestLineShare | Measure::TangleItems => Enforcer::Lint,
        }
    }

    #[must_use]
    pub const fn item_filter(&self) -> Option<&ItemFilter> {
        match self {
            Measure::OpenItems(filter) => Some(filter),
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::GitProcesses
            | Measure::WorkflowWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems
            | Measure::GitStarters => None,
        }
    }

    #[must_use]
    pub const fn window(&self) -> Option<Window> {
        match self {
            Measure::OpenItems(_) => Some(Window::Standing),
            Measure::TestLineShare
            | Measure::TangleItems
            | Measure::GitProcesses
            | Measure::GitStarters => None,
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings => Some(Window::PerChange),
        }
    }
}
