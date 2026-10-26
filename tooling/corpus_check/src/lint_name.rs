use strum::{Display, EnumIter, EnumString, IntoStaticStr};

use const_shape::ConstShape;
use crate_graph::CrateGraph;
use distinct_signatures::DistinctSignatures;
use expression_flow::ExpressionFlow;
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
use task::Policy;
use task::Report;
use task::SourceFile;
use test_budget::TestBudget;
use type_shape::TypeShape;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum LintName {
    ConstShape,
    CrateGraph,
    DistinctSignatures,
    ExpressionFlow,
    FnShape,
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
    TestBudget,
    TypeShape,
}

impl LintName {
    #[must_use]
    pub fn report(self, files: &[SourceFile], policy: &Policy) -> Report {
        match self {
            LintName::ConstShape => ConstShape::report(files, policy),
            LintName::CrateGraph => CrateGraph::report(files, policy),
            LintName::DistinctSignatures => DistinctSignatures::report(files),
            LintName::ExpressionFlow => ExpressionFlow::report(files),
            LintName::FnShape => FnShape::report(files, policy),
            LintName::LiteralNames => LiteralNames::report(files, policy),
            LintName::ManualIteration => ManualIteration::report(files),
            LintName::ModuleNesting => ModuleNesting::report(files, policy),
            LintName::NamedLifetimes => NamedLifetimes::report(files, policy),
            LintName::NamedNumbers => NamedNumbers::report(files, policy),
            LintName::NoComments => NoComments::report(files),
            LintName::NoForwarders => NoForwarders::report(files),
            LintName::NoFreeFns => NoFreeFns::report(files),
            LintName::NoNumbersInBinaries => NoNumbersInBinaries::report(files),
            LintName::NoParameterBags => NoParameterBags::report(files),
            LintName::PrimitiveBoundary => PrimitiveBoundary::report(files, policy),
            LintName::PrivateFns => PrivateFns::report(files, policy),
            LintName::PublicSurface => PublicSurface::report(files),
            LintName::SettingsShape => SettingsShape::report(files),
            LintName::StateGraph => StateGraph::report(files, policy),
            LintName::TestBudget => TestBudget::report(files, policy),
            LintName::TypeShape => TypeShape::report(files),
        }
    }
}
