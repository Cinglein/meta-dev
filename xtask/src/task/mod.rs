mod ci;
mod clippy;
mod coverage;
mod failure;
#[cfg(test)]
mod fixture;
mod lint;
mod prior_policy;
mod ratchet;
mod revision;
mod surfaces;
mod test;
mod usage;

use settings::EmbeddedDefaults;
use strum::{EnumString, IntoStaticStr, VariantNames};
use task::{CrateName, Policy, Workspace};

use crate::task::clippy::Clippy;
use crate::task::coverage::Coverage;
use crate::task::lint::Lint;
use crate::task::surfaces::Surfaces;
use crate::task::test::Test;
use crate::task::usage::Usage;
use affected::{Affected, Changes};
use ci::Ci;
use corpus_check::LintName;
use failure::Failure;
use test_time::TestTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, IntoStaticStr, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Task {
    Affected,
    Baseline,
    Ci,
    Clippy,
    ConstShape,
    CrateGraph,
    DistinctSignatures,
    EnumPredicates,
    ExpressionFlow,
    FnShape,
    Lint,
    LiteralNames,
    ManualIteration,
    ModuleNesting,
    NamedLifetimes,
    NamedNumbers,
    NoComments,
    NoForwarders,
    NoFreeFns,
    NoNumbersInBinaries,
    NoParameterBags,
    PrimitiveBoundary,
    PrivateFns,
    PublicSurface,
    SettingsShape,
    StateGraph,
    Surfaces,
    Test,
    TestBudget,
    TestTime,
    ThinMain,
    TypeShape,
    VecOutParams,
}

impl Task {
    #[must_use]
    pub fn usage() -> Failure {
        Failure::from(task::Failure::from(Usage))
    }

    pub fn run(self, flags: impl Iterator<Item = String>) -> Result<(), Failure> {
        let Ok(tool) = env!("CARGO_PKG_NAME").parse::<CrateName>();
        let workspace = Workspace::locate(tool)?;
        let policy = Policy::defaults();
        let selection = Coverage::from_flags(flags, &workspace)?;
        match self {
            Task::Affected => {
                Ok(Affected::selection(&workspace, Changes::SinceMain).map(|reached| println!("{reached}"))?)
            }
            Task::Baseline => {
                Lint::shrink(&workspace, &policy).map(|kept| println!("baseline: {} violations", kept.total()))
            }
            Task::Ci => Ci::run(&workspace, &selection, &policy),
            Task::Clippy => Ok(Clippy::run(&workspace, &selection, &policy)?),
            Task::Lint => Lint::run(&workspace, &policy),
            Task::Surfaces => Surfaces::announce(workspace.root(), &policy),
            Task::Test => Ok(Test::run(&workspace, &selection, &policy)?),
            Task::TestTime => Ok(TestTime::run(&workspace, &selection, &policy)?),
            lint => Ok(<&'static str>::from(lint)
                .parse::<LintName>()
                .map_or(Ok(()), |lint| lint.report(&workspace.source_files()?, &policy).verdict())?),
        }
    }
}
