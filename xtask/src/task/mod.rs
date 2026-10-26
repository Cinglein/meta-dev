mod ci;
mod clippy;
mod coverage;
mod failure;
mod lint;
mod test;
mod usage;

use settings::EmbeddedDefaults;
use strum::{EnumString, VariantNames};

use crate::task::clippy::Clippy;
use crate::task::coverage::Coverage;
use crate::task::lint::Lint;
use crate::task::test::Test;
use crate::task::usage::Usage;
use affected::Affected;
use affected::Changes;
use ci::Ci;
use const_shape::ConstShape;
use distinct_signatures::DistinctSignatures;
use expression_flow::ExpressionFlow;
use failure::Failure;
use fn_shape::FnShape;
use literal_names::LiteralNames;
use manual_iteration::ManualIteration;
use meta_lints::Project;
use module_nesting::ModuleNesting;
use named_lifetimes::NamedLifetimes;
use named_numbers::NamedNumbers;
use no_comments::NoComments;
use no_forwarders::NoForwarders;
use no_free_fns::NoFreeFns;
use no_numbers_in_binaries::NoNumbersInBinaries;
use no_parameter_bags::NoParameterBags;
use primitive_boundary::PrimitiveBoundary;
use private_fns::PrivateFns;
use public_surface::PublicSurface;
use settings_shape::SettingsShape;
use state_graph::StateGraph;
use task::CrateName;
use task::Policy;
use task::Workspace;
use test_budget::TestBudget;
use test_time::TestTime;
use type_shape::TypeShape;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Task {
    Affected,
    Baseline,
    Ci,
    Clippy,
    ConstShape,
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
    Test,
    TestBudget,
    TestTime,
    TypeShape,
}

impl Task {
    #[must_use]
    pub fn usage() -> Failure {
        Failure::from(task::Failure::from(Usage))
    }

    pub fn run(self, flags: impl Iterator<Item = String>) -> Result<(), Failure> {
        let Ok(tool) = env!("CARGO_PKG_NAME").parse::<CrateName>();
        let workspace = Workspace::locate(tool)?;
        let project = Project::locate(workspace.root())?;
        let policy = Policy::defaults();
        let selection = Coverage::from_flags(flags, &workspace)?;
        match self {
            Task::Affected => {
                Ok(Affected::selection(&workspace, Changes::SinceMain).map(|reached| println!("{reached}"))?)
            }
            Task::Baseline => Ok(project.lints().shrink_baseline().map(|tally| print!("{tally}"))?),
            Task::Ci => Ci::run(&workspace, &selection, &policy, &project),
            Task::Clippy => Ok(Clippy::run(&workspace, &selection, &policy)?),
            Task::Graph => Ok(project.lints().crate_graphs().map(|report| print!("{report}"))?),
            Task::ConstShape => Ok(ConstShape::report(&workspace.source_files()?, &policy).verdict()?),
            Task::DistinctSignatures => Ok(DistinctSignatures::report(&workspace.source_files()?).verdict()?),
            Task::ExpressionFlow => Ok(ExpressionFlow::report(&workspace.source_files()?).verdict()?),
            Task::FnShape => Ok(FnShape::report(&workspace.source_files()?, &policy).verdict()?),
            Task::Lint => Lint::run(&workspace, &policy, &project),
            Task::LiteralNames => Ok(LiteralNames::report(&workspace.source_files()?, &policy).verdict()?),
            Task::ManualIteration => Ok(ManualIteration::report(&workspace.source_files()?).verdict()?),
            Task::ModuleNesting => Ok(ModuleNesting::report(&workspace.source_files()?, &policy).verdict()?),
            Task::NamedLifetimes => Ok(NamedLifetimes::report(&workspace.source_files()?, &policy).verdict()?),
            Task::NamedNumbers => Ok(NamedNumbers::report(&workspace.source_files()?, &policy).verdict()?),
            Task::NoComments => Ok(NoComments::report(&workspace.source_files()?).verdict()?),
            Task::NoForwarders => Ok(NoForwarders::report(&workspace.source_files()?).verdict()?),
            Task::NoFreeFns => Ok(NoFreeFns::report(&workspace.source_files()?).verdict()?),
            Task::NoNumbersInBinaries => Ok(NoNumbersInBinaries::report(&workspace.source_files()?).verdict()?),
            Task::NoParameterBags => Ok(NoParameterBags::report(&workspace.source_files()?).verdict()?),
            Task::PrimitiveBoundary => Ok(PrimitiveBoundary::report(&workspace.source_files()?, &policy).verdict()?),
            Task::PrivateFns => Ok(PrivateFns::report(&workspace.source_files()?, &policy).verdict()?),
            Task::PublicSurface => Ok(PublicSurface::report(&workspace.source_files()?).verdict()?),
            Task::SettingsShape => Ok(SettingsShape::report(&workspace.source_files()?).verdict()?),
            Task::StateGraph => Ok(StateGraph::report(&workspace.source_files()?, &policy).verdict()?),
            Task::Test => Ok(Test::run(&workspace, &selection, &policy)?),
            Task::TestBudget => Ok(TestBudget::report(&workspace.source_files()?, &policy).verdict()?),
            Task::TestTime => Ok(TestTime::run(&workspace, &selection, &policy)?),
            Task::TypeShape => Ok(TypeShape::report(&workspace.source_files()?).verdict()?),
        }
    }
}
