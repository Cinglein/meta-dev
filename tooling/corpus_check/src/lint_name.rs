use strum::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::enforcement::Enforcement;
use const_shape::ConstShape;
use crate_graph::GraphReport;
use distinct_signatures::DistinctSignatures;
use enum_predicates::EnumPredicates;
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
use repeated_methods::RepeatedMethods;
use settings_shape::SettingsShape;
use state_graph::StateGraph;
use task::Policy;
use task::Report;
use task::SourceFile;
use test_budget::TestBudget;
use thin_main::ThinMain;
use type_shape::TypeShape;
use vec_out_params::VecOutParams;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum LintName {
    ConstShape,
    CrateGraph,
    DistinctSignatures,
    EnumPredicates,
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
    RepeatedMethods,
    SettingsShape,
    StateGraph,
    TestBudget,
    ThinMain,
    TypeShape,
    VecOutParams,
}

impl LintName {
    #[must_use]
    pub const fn enforcement(self) -> Enforcement {
        match self {
            LintName::RepeatedMethods => Enforcement::Listed,
            LintName::ConstShape
            | LintName::CrateGraph
            | LintName::DistinctSignatures
            | LintName::EnumPredicates
            | LintName::ExpressionFlow
            | LintName::FnShape
            | LintName::LiteralNames
            | LintName::ManualIteration
            | LintName::ModuleNesting
            | LintName::NamedLifetimes
            | LintName::NamedNumbers
            | LintName::NoComments
            | LintName::NoForwarders
            | LintName::NoFreeFns
            | LintName::NoNumbersInBinaries
            | LintName::NoParameterBags
            | LintName::PrimitiveBoundary
            | LintName::PrivateFns
            | LintName::PublicSurface
            | LintName::SettingsShape
            | LintName::StateGraph
            | LintName::TestBudget
            | LintName::ThinMain
            | LintName::TypeShape
            | LintName::VecOutParams => Enforcement::Ratchet,
        }
    }

    #[must_use]
    pub fn report(self, files: &[SourceFile], graphs: &GraphReport, policy: &Policy) -> Report {
        match self {
            LintName::ConstShape => ConstShape::report(files, policy),
            LintName::CrateGraph => graphs.report(policy),
            LintName::DistinctSignatures => DistinctSignatures::report(files),
            LintName::EnumPredicates => EnumPredicates::report(files),
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
            LintName::RepeatedMethods => RepeatedMethods::report(files, policy),
            LintName::SettingsShape => SettingsShape::report(files),
            LintName::StateGraph => StateGraph::report(files, policy),
            LintName::TestBudget => TestBudget::report(files, policy),
            LintName::ThinMain => ThinMain::report(files),
            LintName::TypeShape => TypeShape::report(files),
            LintName::VecOutParams => VecOutParams::report(files),
        }
    }
}
