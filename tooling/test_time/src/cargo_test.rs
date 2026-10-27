use super::runner_config::RunnerConfig;
use task::{CargoArgument, CargoTool, CommandLine, Policy, Selection};

#[derive(Debug)]
pub(super) struct CargoTest {
    line: CommandLine,
}

impl CargoTest {
    pub(super) fn new(line: CommandLine, selection: &Selection, policy: &Policy) -> CargoTest {
        CargoTest {
            line: line
                .argument(CargoArgument::Test)
                .arguments(selection.arguments::<CargoTool>())
                .argument(CargoArgument::AllFeatures)
                .argument(CargoArgument::Jobs)
                .argument(policy.compiler_tasks())
                .argument(CargoArgument::NoFailFast),
        }
    }

    pub(super) fn through(self, runner: &RunnerConfig) -> CommandLine {
        self.line
            .argument(CargoArgument::Tests)
            .argument(CargoArgument::Config)
            .argument(runner)
    }

    pub(super) fn doctests(self) -> CommandLine {
        self.line.argument(CargoArgument::Doc)
    }
}
