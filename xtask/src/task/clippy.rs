use task::Failure;
use task::Workspace;
use task::{CargoArgument, Program};

pub struct Clippy;

impl Clippy {
    pub fn run(workspace: &Workspace) -> Result<(), Failure> {
        workspace
            .line(Program::Cargo)
            .argument(CargoArgument::Clippy)
            .argument(CargoArgument::Workspace)
            .argument(CargoArgument::AllTargets)
            .argument(CargoArgument::AllFeatures)
            .argument(CargoArgument::ToolArguments)
            .argument(CargoArgument::Deny)
            .argument(CargoArgument::Warnings)
            .run()
    }
}
