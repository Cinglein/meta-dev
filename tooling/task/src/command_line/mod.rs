mod cargo_argument;
mod git_argument;
mod mutants_argument;
mod program;
#[cfg(test)]
mod stderr;
mod stdout;
mod stream;

pub use cargo_argument::CargoArgument;
pub use git_argument::GitArgument;
pub use mutants_argument::MutantsArgument;
pub use program::Program;
#[cfg(test)]
pub use stderr::Stderr;
pub use stdout::Stdout;
pub use stream::Stream;

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};

use meta_run::{Environment, SpawnTrace};

use crate::failure::Failure;
use crate::site::Site;

#[derive(Clone, Debug)]
pub struct CommandLine {
    program: Program,
    arguments: Vec<String>,
    directory: Option<PathBuf>,
    variables: BTreeMap<&'static str, OsString>,
    inherited: Option<BTreeMap<String, OsString>>,
}

impl CommandLine {
    #[must_use]
    pub fn new(program: Program) -> CommandLine {
        CommandLine {
            program,
            arguments: Vec::new(),
            directory: None,
            variables: BTreeMap::new(),
            inherited: None,
        }
    }

    #[must_use]
    pub fn inheriting_only(self, names: impl IntoIterator<Item = impl AsRef<str>>) -> CommandLine {
        self.inheriting_only_from(&CommandLine::ambient(), names)
    }

    #[must_use]
    pub fn ambient() -> BTreeMap<String, OsString> {
        env::vars_os()
            .filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value)))
            .collect()
    }

    #[must_use]
    pub fn inheriting_only_from(
        self,
        ambient: &BTreeMap<String, OsString>,
        names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> CommandLine {
        let traced = SpawnTrace::TRACED.map(str::to_owned);
        let kept = names.into_iter().map(|name| name.as_ref().to_owned());
        CommandLine {
            inherited: Some(
                kept.chain(traced)
                    .filter_map(|name| {
                        ambient
                            .get(&name)
                            .map(|value| (name.clone(), value.clone()))
                    })
                    .collect(),
            ),
            ..self
        }
    }

    #[must_use]
    pub fn environment(mut self, name: &'static str, value: impl Into<OsString>) -> CommandLine {
        self.variables.insert(name, value.into());
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
        self.finished(self.command())
    }

    pub fn run_holding(&self, held: File) -> Result<(), Failure> {
        let mut command = self.command();
        command.stdin(held);
        self.finished(command)
    }

    fn finished(&self, mut command: Command) -> Result<(), Failure> {
        let status = command.status();
        self.started(&command, status)?
            .success()
            .then_some(())
            .ok_or_else(|| Failure::Command(self.to_string()))
    }

    pub fn spawn(&self) -> Result<Child, Failure> {
        let mut command = self.command();
        let child = command.spawn();
        self.started(&command, child)
    }

    fn started<Started>(
        &self,
        command: &Command,
        started: io::Result<Started>,
    ) -> Result<Started, Failure> {
        let started = started.map_err(|error| Failure::Printed {
            command: self.to_string(),
            printed: error.to_string(),
        })?;
        SpawnTrace::within(&Environment::current())
            .noted(command.get_program())
            .map_err(|error| Failure::Io {
                site: Site::Workspace,
                error,
            })?;
        Ok(started)
    }

    pub fn streamed(&self) -> Result<String, Failure> {
        let output = self
            .command()
            .stderr(Stdio::inherit())
            .output()
            .map_err(|_| Failure::Command(self.to_string()))?;
        let printed = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() {
            Ok(printed)
        } else {
            Err(Failure::Printed {
                command: self.to_string(),
                printed,
            })
        }
    }

    pub fn captured<S: Stream>(&self) -> Result<String, Failure> {
        self.output().map(|output| S::pick(&output))
    }

    fn output(&self) -> Result<Output, Failure> {
        let mut command = self.command();
        let output = command.output();
        let output = self.started(&command, output)?;
        if output.status.success() {
            Ok(output)
        } else {
            let printed: Vec<String> = [&output.stdout, &output.stderr]
                .into_iter()
                .map(|stream| String::from_utf8_lossy(stream).trim().to_owned())
                .filter(|text| !text.is_empty())
                .collect();
            Err(Failure::Printed {
                command: self.to_string(),
                printed: printed.join("\n"),
            })
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(self.program.to_string());
        command.args(&self.arguments);
        self.directory.iter().for_each(|directory| {
            command.current_dir(directory);
        });
        self.inherited.iter().for_each(|inherited| {
            command.env_clear().envs(inherited);
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
    use super::{CargoArgument, CommandLine, Program, Stderr, Stdout};
    use crate::failure::Failure;
    use std::fs::File;
    use std::path::{Path, PathBuf};
    const SHOWN: &str = "sh fmt --all";
    const PROGRAMS: [&str; 4] = ["pwd", "true", "false", "sh"];
    const TO_STDERR: [&str; 2] = ["-c", "echo said >&2"];
    const SAID: &str = "said\n";
    const SHELL: &str = "sh";
    const SHOW_TARGET: [&str; 2] = ["-c", "printf %s \"$CARGO_TARGET_DIR$HOME\""];
    const TARGET: &str = "shared/target";
    const VARIABLE: &str = "CARGO_TARGET_DIR";
    const INHERITED: &str = "PATH";
    const MANIFEST: &str = "Cargo.toml";
    const EMPTY: &str = "/dev/null";
    const READS_PACKAGE: [&str; 2] = ["-c", "read line && [ \"$line\" = \"[package]\" ]"];
    const PRINTS_AND_FAILS: [&str; 2] =
        ["-c", "echo out | tr o O; echo err | tr e E >&2; exit 101"];
    const PRINTS: &str = "echo streamed";
    const PRINTS_THEN_FAILS: &str = "echo streamed; exit 1";
    const STREAMED: &str = "streamed\n";
    const OUT: &str = "Out";
    const ERR: &str = "Err";

    #[test]
    fn a_command_runs_in_its_directory_and_its_output_is_captured_and_shown_when_it_fails() {
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
        let failure = executable(shell)
            .arguments(PRINTS_AND_FAILS)
            .captured::<Stdout>()
            .unwrap_err()
            .to_string();
        let printed = failure.contains(OUT) && failure.ends_with(ERR);
        assert!(complained.ok().as_deref() == Some(SAID) && printed);
    }

    #[test]
    fn a_command_prints_its_arguments_sees_only_its_given_environment_and_holds_its_input() {
        let shell = || CommandLine::new(Program::Executable(SHELL.into()));
        let line = shell()
            .argument(CargoArgument::Fmt)
            .arguments([CargoArgument::All]);
        let shown = shell()
            .arguments(SHOW_TARGET)
            .inheriting_only([INHERITED])
            .environment(VARIABLE, PathBuf::from(TARGET))
            .captured::<Stdout>();
        assert_eq!(line.to_string(), SHOWN);
        assert_eq!(shown.ok().as_deref(), Some(TARGET));
        let manifest = || File::open(Path::new(env!("CARGO_MANIFEST_DIR")).join(MANIFEST)).unwrap();
        let reads = shell().arguments(READS_PACKAGE);
        let empty = File::open(EMPTY).unwrap();
        assert!(reads.run_holding(manifest()).is_ok() && reads.run_holding(empty).is_err());
    }

    #[test]
    fn a_streamed_command_returns_its_stdout_whether_it_succeeds_or_fails() {
        let shell = |script: &str| {
            CommandLine::new(Program::Executable(SHELL.into())).arguments(["-c", script])
        };
        let failed = shell(PRINTS_THEN_FAILS).streamed();
        assert_eq!(shell(PRINTS).streamed().ok().as_deref(), Some(STREAMED));
        assert!(matches!(failed, Err(Failure::Printed { printed, .. }) if printed == STREAMED));
    }
}
