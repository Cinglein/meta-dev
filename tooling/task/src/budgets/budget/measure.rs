mod build_start;
mod enforcer;
mod item_filter;
mod mutation_job;

use serde::{Deserialize, Serialize};
use strum::Display;

use super::window::Window;
use crate::domains::ScopeKind;
pub use build_start::BuildStart;
pub use enforcer::Enforcer;
pub use item_filter::ItemFilter;
pub use meta_budget::Unit;
pub use mutation_job::MutationJob;

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
    #[strum(to_string = "build_wall_time {0}")]
    BuildWallTime(BuildStart),
    MutantPrice,
    LintFindings,
    TestLineShare,
    TangleItems,
    GitStarters,
    #[strum(to_string = "open_items {0}")]
    OpenItems(ItemFilter),
    EffectShare,
    EffectDispersion,
    RepeatedShapes,
}

impl Measure {
    #[must_use]
    pub const fn unit(&self) -> Unit {
        match self {
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_)
            | Measure::BuildWallTime(_) => Unit::Duration,
            Measure::MutantPrice | Measure::TestLineShare | Measure::EffectShare => Unit::Share,
            Measure::TangleItems
            | Measure::OpenItems(_)
            | Measure::EffectDispersion
            | Measure::RepeatedShapes
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
            Measure::WorkflowWallTime(_) | Measure::BuildWallTime(_) | Measure::MutantPrice => {
                Some(ScopeKind::Workflows)
            }
            Measure::TangleItems | Measure::EffectShare | Measure::EffectDispersion => {
                Some(ScopeKind::Crates)
            }
            Measure::LintFindings | Measure::TestLineShare | Measure::RepeatedShapes => {
                Some(ScopeKind::Paths)
            }
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
            | Measure::BuildWallTime(_)
            | Measure::OpenItems(_) => Enforcer::Checker,
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => Enforcer::MutationJobs,
            Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems
            | Measure::EffectShare
            | Measure::EffectDispersion
            | Measure::RepeatedShapes => Enforcer::Lint,
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
            | Measure::BuildWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems
            | Measure::EffectShare
            | Measure::EffectDispersion
            | Measure::RepeatedShapes
            | Measure::GitStarters => None,
        }
    }

    #[must_use]
    pub const fn build_start(&self) -> Option<&BuildStart> {
        match self {
            Measure::BuildWallTime(start) => Some(start),
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::GitProcesses
            | Measure::WorkflowWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings
            | Measure::TestLineShare
            | Measure::TangleItems
            | Measure::GitStarters
            | Measure::OpenItems(_)
            | Measure::EffectShare
            | Measure::EffectDispersion
            | Measure::RepeatedShapes => None,
        }
    }

    #[must_use]
    pub const fn window(&self) -> Option<Window> {
        match self {
            Measure::OpenItems(_) => Some(Window::Standing),
            Measure::TestLineShare
            | Measure::TangleItems
            | Measure::EffectShare
            | Measure::EffectDispersion
            | Measure::RepeatedShapes
            | Measure::GitProcesses
            | Measure::GitStarters => None,
            Measure::TestWallTime
            | Measure::TestCpuTime
            | Measure::SlowestTest
            | Measure::WorkflowWallTime(_)
            | Measure::BuildWallTime(_)
            | Measure::MutantPrice
            | Measure::LintFindings => Some(Window::PerChange),
        }
    }
}
