mod binary_files;
mod const_shape;
mod distinct_signatures;
mod enum_predicates;
mod expression_flow;
mod findings;
mod fn_shape;
mod literal_names;
mod manual_iteration;
mod module_nesting;
mod name;
mod named_lifetimes;
mod no_comments;
mod no_forwarders;
mod no_free_fns;
mod no_numbers_in_binaries;
mod no_parameter_bags;
mod ownership;
mod primitive_boundary;
mod private_fns;
mod public_surface;
mod report;
mod state_graph;
mod test_budget;
mod type_shape;
mod vec_out_params;
mod violation;

pub use findings::Findings;
pub use ownership::OwnershipRule;

use strum::{EnumIter, IntoStaticStr};

use crate::lint_config::LintConfig;
use crate::rule::const_shape::ConstShape;
use crate::rule::distinct_signatures::DistinctSignatures;
use crate::rule::enum_predicates::EnumPredicates;
use crate::rule::expression_flow::ExpressionFlow;
use crate::rule::fn_shape::FnShape;
use crate::rule::literal_names::LiteralNames;
use crate::rule::manual_iteration::ManualIteration;
use crate::rule::module_nesting::ModuleNesting;
use crate::rule::named_lifetimes::NamedLifetimes;
use crate::rule::no_comments::NoComments;
use crate::rule::no_forwarders::NoForwarders;
use crate::rule::no_free_fns::NoFreeFns;
use crate::rule::no_numbers_in_binaries::NoNumbersInBinaries;
use crate::rule::no_parameter_bags::NoParameterBags;
use crate::rule::ownership::Ownership;
use crate::rule::primitive_boundary::PrimitiveBoundary;
use crate::rule::private_fns::PrivateFns;
use crate::rule::public_surface::PublicSurface;
use crate::rule::report::Report;
use crate::rule::state_graph::StateGraph;
use crate::rule::test_budget::TestBudget;
use crate::rule::type_shape::TypeShape;
use crate::rule::vec_out_params::VecOutParams;
use crate::source_file::SourceFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Rule {
    ConstShape,
    DistinctSignatures,
    EnumPredicates,
    ExpressionFlow,
    FnShape,
    LiteralNames,
    ManualIteration,
    ModuleNesting,
    NamedLifetimes,
    NoComments,
    NoForwarders,
    NoFreeFns,
    NoNumbersInBinaries,
    NoParameterBags,
    Ownership,
    PrimitiveBoundary,
    PrivateFns,
    PublicSurface,
    StateGraph,
    TestBudget,
    TypeShape,
    VecOutParams,
}

impl Rule {
    pub fn report(self, files: &[SourceFile], config: &LintConfig) -> Report {
        match self {
            Rule::ConstShape => ConstShape::report(files),
            Rule::DistinctSignatures => DistinctSignatures::report(files),
            Rule::EnumPredicates => EnumPredicates::report(files),
            Rule::ExpressionFlow => ExpressionFlow::report(files),
            Rule::FnShape => FnShape::report(files),
            Rule::LiteralNames => LiteralNames::report(files),
            Rule::ManualIteration => ManualIteration::report(files),
            Rule::ModuleNesting => ModuleNesting::report(files),
            Rule::NamedLifetimes => NamedLifetimes::report(files),
            Rule::NoComments => NoComments::report(files),
            Rule::NoForwarders => NoForwarders::report(files),
            Rule::NoFreeFns => NoFreeFns::report(files),
            Rule::NoNumbersInBinaries => NoNumbersInBinaries::report(files),
            Rule::NoParameterBags => NoParameterBags::report(files),
            Rule::Ownership => Ownership::report(files, config.ownership()),
            Rule::PrimitiveBoundary => PrimitiveBoundary::report(files),
            Rule::PrivateFns => PrivateFns::report(files),
            Rule::PublicSurface => PublicSurface::report(files),
            Rule::StateGraph => StateGraph::report(files),
            Rule::TestBudget => TestBudget::report(files),
            Rule::TypeShape => TypeShape::report(files),
            Rule::VecOutParams => VecOutParams::report(files),
        }
    }
}
