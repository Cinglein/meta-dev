mod cargo_argument;
mod program;
mod stdout;
mod stream;

pub use cargo_argument::CargoArgument;
pub use program::Program;
pub use stdout::Stdout;
pub use stream::Stream;

use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::failure::Failure;

#[derive(Clone, Debug)]
pub struct CommandLine {
    program: Program,
    arguments: Vec<String>,
    directory: Option<PathBuf>,
}

impl CommandLine {
    #[must_use]
    pub fn new(program: Program) -> CommandLine {
        CommandLine {
            program,
            arguments: Vec::new(),
            directory: None,
        }
    }

    #[must_use]
    pub fn in_directory(self, directory: PathBuf) -> CommandLine {
        CommandLine {
            directory: Some(directory),
            ..self
        }
    }

    #[must_use]
    pub fn argument(mut self, argument: impl fmt::Display) -> CommandLine {
        self.arguments.push(argument.to_string());
        self
    }

    #[must_use]
    pub fn arguments(self, arguments: impl IntoIterator<Item = impl fmt::Display>) -> CommandLine {
        arguments.into_iter().fold(self, CommandLine::argument)
    }

    pub fn run(&self) -> Result<(), Failure> {
        self.command()
            .status()
            .ok()
            .filter(std::process::ExitStatus::success)
            .map(|_| ())
            .ok_or_else(|| Failure::Command(self.to_string()))
    }

    pub fn captured<S: Stream>(&self) -> Result<String, Failure> {
        self.output().map(|output| S::pick(&output))
    }

    fn output(&self) -> Result<Output, Failure> {
        let output = self
            .command()
            .output()
            .map_err(|_| Failure::Command(self.to_string()))?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(Failure::Command(format!(
                "{self}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(self.program.to_string());
        command.args(&self.arguments);
        self.directory.iter().for_each(|directory| {
            command.current_dir(directory);
        });
        command
    }
}

impl fmt::Display for CommandLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.program, self.arguments.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::{CargoArgument, CommandLine, Program, Stdout};
    use std::path::PathBuf;
    const GIT: &str = "git";
    const SHOWN: &str = "git fmt --all";
    const PROGRAMS: [&str; 3] = ["pwd", "true", "false"];

    #[test]
    fn a_command_line_prints_its_program_and_arguments_in_order() {
        let line = CommandLine::new(Program::Executable(GIT.into()))
            .argument(CargoArgument::Fmt)
            .arguments([CargoArgument::All]);
        assert_eq!(line.to_string(), SHOWN);
    }

    #[test]
    fn a_command_runs_in_its_directory_and_its_output_is_captured() {
        let [pwd, succeeds, fails] = PROGRAMS;
        let executable = |name: &str| CommandLine::new(Program::Executable(name.into()));
        let directory = std::env::temp_dir();
        let shown = executable(pwd)
            .in_directory(directory.clone())
            .captured::<Stdout>()
            .unwrap();
        assert_eq!(
            PathBuf::from(shown.trim()).canonicalize().ok(),
            directory.canonicalize().ok()
        );
        assert!(executable(succeeds).run().is_ok() && executable(fails).run().is_err());
    }
}
