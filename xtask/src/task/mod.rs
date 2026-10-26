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
use strum::{EnumIter, EnumString, IntoStaticStr, VariantNames};
use task::{Policy, Report, SourceFile, Workspace};

use crate::task::clippy::Clippy;
use crate::task::lint::Lint;
use crate::task::surfaces::Surfaces;
use crate::task::test::Test;
use crate::task::usage::Usage;
use ci::Ci;
use const_shape::ConstShape;
use crate_graph::CrateGraph;
use distinct_signatures::DistinctSignatures;
use expression_flow::ExpressionFlow;
use failure::Failure;
use fn_shape::FnShape;
use literal_names::LiteralNames;
use manual_iteration::ManualIteration;
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
use test_budget::TestBudget;
use type_shape::TypeShape;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, EnumString, IntoStaticStr, VariantNames)]
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
            lint => Ok(lint.report(&workspace.source_files()?, &policy).map_or(Ok(()), Report::verdict)?),
        }
    }

    fn report(self, files: &[SourceFile], policy: &Policy) -> Option<Report> {
        match self {
            Task::ConstShape => Some(ConstShape::report(files, policy)),
            Task::CrateGraph => Some(CrateGraph::report(files, policy)),
            Task::DistinctSignatures => Some(DistinctSignatures::report(files)),
            Task::ExpressionFlow => Some(ExpressionFlow::report(files)),
            Task::FnShape => Some(FnShape::report(files, policy)),
            Task::LiteralNames => Some(LiteralNames::report(files, policy)),
            Task::ManualIteration => Some(ManualIteration::report(files)),
            Task::ModuleNesting => Some(ModuleNesting::report(files, policy)),
            Task::NamedLifetimes => Some(NamedLifetimes::report(files, policy)),
            Task::NamedNumbers => Some(NamedNumbers::report(files, policy)),
            Task::NoComments => Some(NoComments::report(files)),
            Task::NoForwarders => Some(NoForwarders::report(files)),
            Task::NoFreeFns => Some(NoFreeFns::report(files)),
            Task::NoNumbersInBinaries => Some(NoNumbersInBinaries::report(files)),
            Task::NoParameterBags => Some(NoParameterBags::report(files)),
            Task::PrimitiveBoundary => Some(PrimitiveBoundary::report(files, policy)),
            Task::PrivateFns => Some(PrivateFns::report(files, policy)),
            Task::PublicSurface => Some(PublicSurface::report(files)),
            Task::SettingsShape => Some(SettingsShape::report(files)),
            Task::StateGraph => Some(StateGraph::report(files, policy)),
            Task::TestBudget => Some(TestBudget::report(files, policy)),
            Task::TypeShape => Some(TypeShape::report(files)),
            Task::Baseline | Task::Ci | Task::Clippy | Task::Graph | Task::Lint | Task::Surfaces | Task::Test => None,
        }
    }
}
