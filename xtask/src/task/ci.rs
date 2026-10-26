use task::{CargoArgument, Policy, Program, Workspace};

use crate::task::clippy::Clippy;
use crate::task::failure::Failure;
use crate::task::lint::Lint;
use crate::task::surfaces::Surfaces;
use crate::task::test::Test;

#[derive(Debug)]
pub struct Ci;

impl Ci {
    pub fn run(workspace: &Workspace, policy: &Policy) -> Result<(), Failure> {
        let outcome = Self::checks(workspace).map_err(Failure::from).and_then(|()| Lint::run(workspace, policy));
        Surfaces::announce(workspace.root(), policy).unwrap_or_else(|failure| eprintln!("surfaces: {failure}"));
        outcome
    }

    fn checks(workspace: &Workspace) -> Result<(), task::Failure> {
        workspace
            .line(Program::Cargo)
            .argument(CargoArgument::Fmt)
            .argument(CargoArgument::All)
            .argument(CargoArgument::Check)
            .run()?;
        Clippy::run(workspace)?;
        Test::run(workspace)
    }
}
