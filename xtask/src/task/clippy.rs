mod diagnostics;

use diagnostics::Diagnostics;
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
                    .argument(CargoArgument::MessageFormatJson)
                    .argument(CargoArgument::Jobs)
                    .argument(policy.compiler_tasks())
                    .argument(CargoArgument::ToolArguments)
                    .argument(CargoArgument::Deny)
                    .argument(CargoArgument::Warnings)
                    .streamed()
                    .map(|_| ())
                    .map_err(|failure| failure.reading(Diagnostics::reading))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::diagnostics::Diagnostics;

    const SHOWN: &str = "cannot find value";
    const PRINTED: &str = include_str!("../../fixtures/compile-error.jsonl");
    const GOLDEN: &str = include_str!("../../fixtures/compile-error.golden.json");

    #[test]
    fn a_compile_error_is_recorded_with_its_path() {
        let read = Diagnostics::reading(PRINTED);
        let found = read.as_ref().map(|failure| failure.found(&"clippy".into()).to_string());
        assert_eq!(found.as_deref(), Some(GOLDEN));
        assert!(read.is_some_and(|failure| failure.to_string().contains(SHOWN)));
    }
}
