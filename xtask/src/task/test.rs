use task::Failure;
use task::Workspace;
use task::{CargoArgument, Program};

pub struct Test;

impl Test {
    pub fn run(workspace: &Workspace) -> Result<(), Failure> {
        workspace
            .line(Program::Cargo)
            .argument(CargoArgument::Test)
            .argument(CargoArgument::Workspace)
            .argument(CargoArgument::AllFeatures)
            .run()
    }
}
