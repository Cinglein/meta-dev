use task::{CargoArgument, CargoTool, CommandLine, Policy, Selection};

#[derive(Debug)]
pub(super) struct CargoBuild;

impl CargoBuild {
    pub(super) fn examples(
        line: CommandLine,
        selection: &Selection,
        policy: &Policy,
    ) -> CommandLine {
        line.argument(CargoArgument::Build)
            .arguments(selection.arguments::<CargoTool>())
            .argument(CargoArgument::AllFeatures)
            .argument(CargoArgument::Jobs)
            .argument(policy.compiler_tasks())
            .argument(CargoArgument::Examples)
            .argument(CargoArgument::Profile)
            .argument(CargoArgument::Test)
    }
}
