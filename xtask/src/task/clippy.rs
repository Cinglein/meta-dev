use task::Failure;
use task::Policy;
use task::Workspace;
use task::{CargoArgument, Program};
use task::{CargoTool, Selection};

pub struct Clippy;

impl Clippy {
    pub fn run(workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
        selection.reaching().map_or_else(
            || {
                println!("clippy: no crate affected");
                Ok(())
            },
            |selection| {
                workspace
                    .line(Program::Cargo)
                    .argument(CargoArgument::Clippy)
                    .arguments(selection.arguments::<CargoTool>())
                    .argument(CargoArgument::AllTargets)
                    .argument(CargoArgument::AllFeatures)
                    .argument(CargoArgument::Jobs)
                    .argument(policy.compiler_tasks())
                    .argument(CargoArgument::ToolArguments)
                    .argument(CargoArgument::Deny)
                    .argument(CargoArgument::Warnings)
                    .run()
            },
        )
    }
}
