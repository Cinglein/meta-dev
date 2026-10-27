mod enforcer;
mod mutation_job;
mod unit;

use serde::{Deserialize, Serialize};
use strum::Display;

use crate::domains::ScopeKind;
pub use enforcer::Enforcer;
pub use mutation_job::MutationJob;
pub use unit::Unit;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
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
}

impl Measure {
    #[must_use]
    pub const fn unit(self) -> Unit {
        match self {
            Measure::TestWallTime | Measure::SlowestTest | Measure::WorkflowWallTime(_) => {
                Unit::Duration
            }
            Measure::MutantPrice | Measure::TestLineShare => Unit::Share,
            Measure::TangleItems => Unit::Count,
            Measure::LintFindings => Unit::Baseline,
        }
    }

    #[must_use]
    pub const fn scope(self) -> ScopeKind {
        match self {
            Measure::TestWallTime | Measure::SlowestTest => ScopeKind::Tests,
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => ScopeKind::Workflows,
            Measure::TangleItems => ScopeKind::Crates,
            Measure::LintFindings | Measure::TestLineShare => ScopeKind::Paths,
        }
    }

    #[must_use]
    pub const fn enforcer(self) -> Enforcer {
        match self {
            Measure::TestWallTime | Measure::SlowestTest => Enforcer::Checker,
            Measure::WorkflowWallTime(_) | Measure::MutantPrice => Enforcer::MutationJobs,
            Measure::LintFindings | Measure::TestLineShare | Measure::TangleItems => Enforcer::Lint,
        }
    }
}
