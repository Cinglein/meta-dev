mod ci;
mod clippy;
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
use task::{Policy, Workspace};

use crate::task::clippy::Clippy;
use crate::task::lint::Lint;
use crate::task::surfaces::Surfaces;
use crate::task::test::Test;
use crate::task::usage::Usage;
use ci::Ci;
use corpus_check::LintName;
use crate_graph::CrateGraph;
use failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, IntoStaticStr, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Task {
    Baseline,
    Ci,
    Clippy,
    ConstShape,
    CrateGraph,
    DistinctSignatures,
    ExpressionFlow,
    FnShape,
    Graph,
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
    TypeShape,
}

impl Task {
    #[must_use]
    pub fn usage() -> Failure {
        Failure::from(task::Failure::from(Usage))
    }

    pub fn run(self) -> Result<(), Failure> {
        let workspace = Workspace::locate()?;
        let policy = Policy::defaults();
        match self {
            Task::Baseline => {
                Lint::shrink(&workspace, &policy).map(|kept| println!("baseline: {} violations", kept.total()))
            }
            Task::Ci => Ci::run(&workspace, &policy),
            Task::Clippy => Ok(Clippy::run(&workspace)?),
            Task::Graph => Ok(workspace.source_files().map(|files| print!("{}", CrateGraph::graphs(&files)))?),
            Task::Lint => Lint::run(&workspace, &policy),
            Task::Surfaces => Surfaces::announce(workspace.root(), &policy),
            Task::Test => Ok(Test::run(&workspace)?),
            lint => Ok(<&'static str>::from(lint)
                .parse::<LintName>()
                .map_or(Ok(()), |lint| lint.report(&workspace.source_files()?, &policy).verdict())?),
        }
    }
}
