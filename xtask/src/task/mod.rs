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

use std::env;
use std::path::{Path, PathBuf};

use strum::{EnumString, IntoStaticStr, VariantNames};
use task::{Branch, Policy, Program, Selection, Workspace};

use crate::task::clippy::Clippy;
use crate::task::coverage::Coverage;
use crate::task::lint::Lint;
use crate::task::surfaces::Surfaces;
use crate::task::test::Test;
use crate::task::usage::Usage;
use affected::{Affected, Changes};
use ci::Ci;
use corpus_check::LintName;
use crate_graph::CrateGraph;
use failure::Failure;
use mutant_costs::MutantCosts;
use mutation::{Mutants, Scope, ShardPlan, TriggerEvent};
use stacked::Stacked;
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
    Graph,
    Lint,
    LiteralNames,
    ManualIteration,
    ModuleNesting,
    MutantCosts,
    Mutants,
    MutantsFull,
    MutantsDiffPlan,
    MutantsPlan,
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
    Stacked,
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

    pub fn run(self, mut flags: impl Iterator<Item = String>) -> Result<(), Failure> {
        let workspace = Workspace::locate()?;
        let policy = Policy::read(workspace.root())?;
        let selection = match self {
            Task::MutantCosts => Selection::Workspace,
            _ => Affected::selection(&workspace, Coverage::from_flags(flags.by_ref())?, policy.branch(Branch::Base))?,
        };
        let summary = env::var_os(Surfaces::SUMMARY).map(PathBuf::from);
        match self {
            Task::Affected => Ok(Affected::selection(&workspace, Changes::SinceMain, policy.branch(Branch::Base))
                .map(|reached| println!("{reached}"))?),
            Task::Baseline => Lint::shrink(workspace.root(), &workspace.source_files()?, &policy)
                .map(|kept| println!("baseline: {} violations", kept.total())),
            Task::Ci => Ci::run(&workspace, &selection, &policy, summary.as_deref()),
            Task::Clippy => Ok(Clippy::run(&workspace, &selection, &policy)?),
            Task::Graph => Ok(workspace.source_files().map(|files| print!("{}", CrateGraph::graphs(&files)))?),
            Task::Lint => Lint::run(workspace.root(), &workspace.source_files()?, &policy),
            Task::MutantCosts => Ok(MutantCosts::measure(Path::new(&flags.next().ok_or_else(Self::usage)?))
                .map(|costs| println!("{costs}"))?),
            Task::Mutants => Ok(Mutants::run(&workspace, Scope::ChangedLines, &policy)?),
            Task::MutantsFull => Ok(Mutants::run(&workspace, Scope::Workspace, &policy)?),
            Task::MutantsPlan => Ok(ShardPlan::full(
                workspace.line(Program::Cargo),
                &policy,
                TriggerEvent::current().and_then(TriggerEvent::full_run),
            )
            .map(|plan| println!("{plan}"))?),
            Task::MutantsDiffPlan => Ok(ShardPlan::changed_lines(&workspace, &policy).map(|plan| println!("{plan}"))?),
            Task::Stacked => Ok(Stacked::run(&policy)?),
            Task::Surfaces => Surfaces::announce(workspace.root(), &policy, summary.as_deref()),
            Task::Test => Ok(Test::run(&workspace, &selection, &policy)?),
            Task::TestTime => Ok(TestTime::run(&workspace, &selection, &policy)?),
            lint => Ok(<&'static str>::from(lint)
                .parse::<LintName>()
                .map_or(Ok(()), |lint| lint.report(&workspace.source_files()?, &policy).verdict())?),
        }
    }
}
