mod cargo_argument;
mod git_argument;
mod mutants_argument;
mod program;
mod stderr;
mod stdout;
mod stream;

pub use cargo_argument::CargoArgument;
pub use git_argument::GitArgument;
pub use mutants_argument::MutantsArgument;
pub use program::Program;
pub use stderr::Stderr;
pub use stdout::Stdout;
pub use stream::Stream;

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::process::{Child, Command, Output};

use crate::failure::Failure;
use crate::workspace::TargetDirectory;

#[derive(Clone, Debug)]
pub struct CommandLine {
    program: Program,
    arguments: Vec<String>,
    directory: Option<PathBuf>,
    variables: BTreeMap<&'static str, PathBuf>,
}

impl CommandLine {
    const TARGET: &str = "CARGO_TARGET_DIR";

    #[must_use]
    pub fn new(program: Program) -> CommandLine {
        CommandLine {
            program,
            arguments: Vec::new(),
            directory: None,
            variables: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn building_into(self, target: &TargetDirectory) -> CommandLine {
        self.environment(Self::TARGET, target.path().to_path_buf())
    }

    #[must_use]
    pub fn environment(mut self, name: &'static str, value: PathBuf) -> CommandLine {
        self.variables.insert(name, value);
        self
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

    pub fn spawn(&self) -> Result<Child, Failure> {
        self.command()
            .spawn()
            .map_err(|_| Failure::Command(self.to_string()))
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
        command.envs(&self.variables);
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
    use super::{CargoArgument, CommandLine, Program, Stderr, Stdout, TargetDirectory};
    use std::path::PathBuf;
    const GIT: &str = "git";
    const SHOWN: &str = "git fmt --all";
    const PROGRAMS: [&str; 4] = ["pwd", "true", "false", "sh"];
    const TO_STDERR: [&str; 2] = ["-c", "echo said >&2"];
    const SAID: &str = "said\n";
    const SHELL: &str = "sh";
    const SHOW_TARGET: [&str; 2] = ["-c", "printf %s \"$CARGO_TARGET_DIR\""];
    const TARGET: &str = "shared/target";
    const SHARED: &str = r#"{"target_directory": "shared/target"}"#;

    #[test]
    fn a_command_line_prints_its_program_and_arguments_in_order() {
        let line = CommandLine::new(Program::Executable(GIT.into()))
            .argument(CargoArgument::Fmt)
            .arguments([CargoArgument::All]);
        assert_eq!(line.to_string(), SHOWN);
    }

    #[test]
    fn a_command_runs_in_its_directory_and_its_output_is_captured() {
        let [pwd, succeeds, fails, shell] = PROGRAMS;
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
        let complained = executable(shell).arguments(TO_STDERR).captured::<Stderr>();
        assert_eq!(complained.ok().as_deref(), Some(SAID));
    }

    #[test]
    fn a_command_building_into_a_target_sees_it_as_the_cargo_target_dir() {
        let shown = CommandLine::new(Program::Executable(SHELL.into()))
            .arguments(SHOW_TARGET)
            .building_into(&TargetDirectory::from_metadata(SHARED).unwrap())
            .captured::<Stdout>();
        assert_eq!(shown.ok().as_deref(), Some(TARGET));
    }
}
