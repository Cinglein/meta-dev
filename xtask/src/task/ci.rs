use crate::task::clippy::Clippy;
use crate::task::failure::Failure;
use crate::task::lint::Lint;
use crate::task::test::Test;
use meta_lints::Project;
use task::Policy;
use task::Selection;
use task::Workspace;
use task::{CargoArgument, Program};
use test_time::TestTime;

pub struct Ci;

impl Ci {
    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        project: &Project,
    ) -> Result<(), Failure> {
        println!("ci: checking {selection}");
        let outcome = Self::checks(workspace, selection, policy)
            .map_err(Failure::from)
            .and_then(|()| Lint::run(workspace, policy, project));
        project
            .changed_surfaces()
            .and_then(|changed| changed.announce())
            .unwrap_or_else(|failure| eprintln!("surfaces: {failure}"));
        outcome
    }

    fn checks(workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), task::Failure> {
        workspace
            .line(Program::Cargo)
            .argument(CargoArgument::Fmt)
            .argument(CargoArgument::All)
            .argument(CargoArgument::Check)
            .run()?;
        Clippy::run(workspace, selection, policy)?;
        Test::run(workspace, selection, policy)?;
        TestTime::run(workspace, selection, policy)
    }
}
