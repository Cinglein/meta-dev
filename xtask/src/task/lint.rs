use crate::task::failure::Failure;
use const_shape::ConstShape;
use distinct_signatures::DistinctSignatures;
use expression_flow::ExpressionFlow;
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
use task::Policy;
use task::Report;
use task::Workspace;
use test_budget::TestBudget;
use type_shape::TypeShape;

pub struct Lint;

impl Lint {
    pub fn run(workspace: &Workspace, policy: &Policy, project: &Project) -> Result<(), Failure> {
        let files = workspace.source_files()?;
        let reports: Vec<Report> = [
            ConstShape::report(&files, policy),
            DistinctSignatures::report(&files),
            ExpressionFlow::report(&files),
            FnShape::report(&files, policy),
            LiteralNames::report(&files, policy),
            ManualIteration::report(&files),
            ModuleNesting::report(&files, policy),
            NamedLifetimes::report(&files, policy),
            NamedNumbers::report(&files, policy),
            NoComments::report(&files),
            NoForwarders::report(&files),
            NoFreeFns::report(&files),
            NoNumbersInBinaries::report(&files),
            NoParameterBags::report(&files),
            PrimitiveBoundary::report(&files, policy),
            PrivateFns::report(&files, policy),
            PublicSurface::report(&files),
            SettingsShape::report(&files),
            StateGraph::report(&files, policy),
            TestBudget::report(&files, policy),
            TypeShape::report(&files),
        ]
        .into_iter()
        .filter(|report| !report.is_clean())
        .collect();
        if reports.is_empty() {
            println!("lint: {} rust files clean", files.len());
            Ok(project.lints().lint(&project.prior()?).map(|tally| print!("{tally}"))?)
        } else {
            Err(Failure::from(task::Failure::Lint(reports)))
        }
    }
}
