mod adoption;
mod build_directory;
mod ci;
mod clippy;
mod coverage;
mod elegance_report;
mod failure;
#[cfg(test)]
mod fixture;
mod golden_refresh;
mod lint;
mod prior_policy;
mod ratchet;
mod revision;
mod supply;
mod surfaces;
mod test;
mod test_processes;
mod usage;

use std::env;
use std::path::PathBuf;

use strum::{EnumString, IntoStaticStr, VariantNames};
use task::{Branch, Ledger, Policy, Program, Selection, Workspace};

use crate::task::adoption::Adoption;
use crate::task::build_directory::{BuildDirectory, Misplaced, TrackedBuildFiles};
use crate::task::clippy::Clippy;
use crate::task::coverage::Coverage;
use crate::task::elegance_report::EleganceReport;
use crate::task::golden_refresh::GoldenRefresh;
use crate::task::lint::Lint;
use crate::task::supply::Supply;
use crate::task::surfaces::Surfaces;
use crate::task::test::Test;
use crate::task::test_processes::TestProcesses;
use crate::task::usage::Usage;
use affected::{Affected, Changes};
use build_time::BuildTime;
use ci::Ci;
pub use ci::CiStep;
use corpus_check::LintName;
use crate_graph::CrateGraph;
use failure::Failure;
use meta_golden::Reach;
use mutant_costs::MutantCosts;
use mutation::{Mutants, Scope, ShardPlan, TriggerEvent};
use stacked::Stacked;
use test_time::{ProcessCount, TestRunner, TestTime};
use waterfall::Waterfall;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, IntoStaticStr, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Task {
    Adoption,
    Affected,
    Baseline,
    Ci,
    BuildTime,
    Clippy,
    ConstShape,
    CrateGraph,
    Deny,
    DistinctSignatures,
    Elegance,
    EnumPredicates,
    ExpressionFlow,
    FnShape,
    Goldens,
    GoldensOnline,
    Graph,
    Init,
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
    RepeatedMethods,
    Semver,
    SettingsShape,
    Stacked,
    StateGraph,
    Surfaces,
    Test,
    TestBudget,
    TestRunner,
    TestTime,
    ThinMain,
    TypeShape,
    VecOutParams,
    Waterfall,
}

impl Task {
    #[must_use]
    pub fn usage() -> Failure {
        Failure::from(task::Failure::from(Usage))
    }

    pub fn run(self, flags: impl Iterator<Item = String>) -> Result<(), Failure> {
        let flags: Vec<String> = flags.collect();
        match self {
            Task::TestRunner => Ok(TestRunner::run(flags.into_iter())?),
            Task::Ci if flags == [Ci::CLOSE] => Ci::close(),
            Task::Init => Ok(BuildDirectory::read(Workspace::locate()?.root())?.initialized()?),
            Task::Waterfall => Waterfall::shown(flags.into_iter().map(PathBuf::from))?
                .map(|shown| print!("{shown}"))
                .ok_or_else(Self::usage),
            task => task.run_in_workspace(&Workspace::locate()?, flags.into_iter()),
        }
    }

    fn run_in_workspace(self, workspace: &Workspace, mut flags: impl Iterator<Item = String>) -> Result<(), Failure> {
        let policy = Policy::read(workspace.root())?;
        let coverage = match self {
            Task::MutantCosts => Coverage::EVERYTHING,
            _ => Coverage::from_flags(self, flags.by_ref())?,
        };
        let selection = match self {
            Task::MutantCosts => Selection::Workspace,
            _ => Affected::selection(workspace, coverage.changes(), policy.branch(Branch::Base))?,
        };
        let summary = env::var_os(Surfaces::SUMMARY).map(PathBuf::from);
        let mut ledger = Ledger::default();
        policy.deprecated().iter().for_each(|key| {
            eprintln!(
                "policy: {key} is deprecated; read as budget {} ({}); declare that budget instead",
                key.budget_id(),
                key.measure()
            );
        });
        match self {
            Task::Adoption => Adoption::checked(workspace, &policy),
            Task::Affected => Ok(Affected::selection(workspace, &Changes::SinceMain, policy.branch(Branch::Base))
                .map(|reached| println!("{reached}"))?),
            Task::Baseline => Lint::shrink(workspace.root(), &workspace.source_files()?, &policy)
                .map(|kept| println!("baseline: {} violations", kept.total())),
            Task::Ci => Ci::new(summary, coverage.gate(workspace.root(), &policy)?, coverage.log().clone())
                .run(workspace, &selection, &policy),
            Task::BuildTime => Ok(BuildTime::run(workspace, &policy, &mut ledger)
                .and_then(|()| ledger.checked(&policy, &Selection::Workspace))?),
            Task::Clippy => Ok(Clippy::run(workspace, &selection, &policy)?),
            Task::Deny => Supply::deny(workspace, &policy),
            Task::Semver => Supply::semver(workspace, &selection, &policy),
            Task::Goldens => GoldenRefresh::run(workspace, Reach::Offline),
            Task::GoldensOnline => GoldenRefresh::run(workspace, Reach::Online),
            Task::Graph => Ok(workspace.source_files().map(|files| print!("{}", CrateGraph::graphs(&files)))?),
            Task::Elegance => EleganceReport::measured(workspace).map(|report| print!("{report}")),
            Task::Lint => Lint::run(workspace.root(), &workspace.source_files()?, &policy, &mut ledger)
                .and_then(|()| Ok(ledger.checked(&policy, &selection)?)),
            Task::MutantCosts => Ok(MutantCosts::measure(
                &Some(flags.map(PathBuf::from).collect::<Vec<PathBuf>>())
                    .filter(|runs| !runs.is_empty())
                    .ok_or_else(Self::usage)?,
                &policy,
            )
            .inspect(|costs| println!("{costs}"))
            .inspect(|costs| {
                eprintln!("below the sample floor, not priced: {}", costs.below_sample_floor());
                eprintln!("{}", costs.within_tolerance(&policy, TriggerEvent::priced_job(TriggerEvent::current())));
            })?
            .priced_within(&policy, TriggerEvent::priced_job(TriggerEvent::current()))?),
            Task::Mutants => Ok(Mutants::run(workspace, Scope::ChangedLines, &policy)?),
            Task::MutantsFull => Ok(Mutants::run(workspace, Scope::Workspace, &policy)?),
            Task::MutantsPlan => Ok(ShardPlan::full(
                workspace.line(Program::Cargo),
                &policy,
                TriggerEvent::current().and_then(TriggerEvent::full_run),
            )
            .map(|plan| println!("{plan}"))?),
            Task::MutantsDiffPlan => Ok(ShardPlan::changed_lines(workspace, &policy).map(|plan| println!("{plan}"))?),
            Task::Stacked => Ok(Stacked::run(&policy)?),
            Task::Surfaces => Surfaces::announce(workspace.root(), &policy, summary.as_deref()),
            Task::Test => Ok(Test::run(workspace, &selection, &policy)?),
            Task::TestTime => match env::var_os(ProcessCount::SWITCH) {
                Some(_) => TestProcesses::run(workspace, &selection, &policy, &mut ledger),
                None => Ok(TestTime::run(workspace, &selection, &policy, &mut ledger)?),
            }
            .and_then(|()| Ok(ledger.checked(&policy, &selection)?)),
            lint => Ok(<&'static str>::from(lint).parse::<LintName>().map_or(Ok(()), |lint| {
                let files = workspace.source_files()?;
                lint.report(&files, &CrateGraph::graphs(&files), &policy).verdict()
            })?),
        }
    }
}
