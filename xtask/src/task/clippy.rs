mod cargo_line;
mod code;
mod code_name;
mod diagnostics;
mod level;
mod message;
mod span;

use diagnostics::Diagnostics;
use task::Failure;
use task::Policy;
use task::Stdout;
use task::Workspace;
use task::{CargoArgument, Program};
use task::{CargoTool, Selection};

pub struct Clippy;

impl Clippy {
    fn diagnosed(failure: Failure) -> Failure {
        match failure {
            Failure::Printed { command, printed } => {
                let diagnostics = Diagnostics::parse(&printed);
                if diagnostics.is_empty() {
                    Failure::Printed { command, printed }
                } else {
                    Failure::Found {
                        shown: format!("{command} failed:\n{}", diagnostics.rendered()),
                        found: diagnostics.found(),
                    }
                }
            }
            other => other,
        }
    }

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
                    .captured::<Stdout>()
                    .map(|_| ())
                    .map_err(Self::diagnosed)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::diagnostics::Diagnostics;

    const PRINTED: &str = include_str!("../../fixtures/compile-error.jsonl");
    const GOLDEN: &str = include_str!("../../fixtures/compile-error.golden.json");

    #[test]
    fn a_compile_error_is_recorded_with_its_path() {
        assert_eq!(Diagnostics::parse(PRINTED).found().to_string(), GOLDEN);
    }
}
