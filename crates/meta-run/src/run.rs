pub(crate) mod spawned;

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::command_line::CommandLine;
use crate::env_policy::EnvPolicy;
use crate::environment::Environment;
use crate::kill_scope::KillScope;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use spawned::Spawned;

#[derive(Debug)]
pub struct Run {
    command: Command,
    scope: KillScope,
    input: Option<String>,
}

impl Run {
    const GIT: &'static str = "git";
    const DIRECTORY: &'static str = "-C";

    #[must_use]
    pub fn new(program: impl AsRef<OsStr>, env: EnvPolicy) -> Self {
        Self::inheriting(program, env, Environment::current())
    }

    #[must_use]
    pub fn inheriting(program: impl AsRef<OsStr>, env: EnvPolicy, inherited: Environment) -> Self {
        let mut command = Command::new(program);
        env.applied(inherited).exported(&mut command);
        Self { command, scope: KillScope::Group, input: None }
    }

    #[must_use]
    pub fn git(dir: &Path) -> Self {
        Self::new(Self::GIT, EnvPolicy::NoRepoVars).arg(Self::DIRECTORY).arg(dir)
    }

    #[must_use]
    pub fn arg(self, word: impl AsRef<OsStr>) -> Self {
        self.args([word])
    }

    #[must_use]
    pub fn args<Word: AsRef<OsStr>>(mut self, words: impl IntoIterator<Item = Word>) -> Self {
        self.command.args(words);
        self
    }

    #[must_use]
    pub fn dir(mut self, dir: &Path) -> Self {
        self.command.current_dir(dir);
        self
    }

    #[must_use]
    pub fn var(mut self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.command.env(name, value);
        self
    }

    #[must_use]
    pub fn fed(self, input: String) -> Self {
        Self { input: Some(input), ..self }
    }

    #[must_use]
    pub fn scoped(self, scope: KillScope) -> Self {
        Self { scope, ..self }
    }

    pub fn ran(self, deadline: Instant) -> Result<Ran, RunFailure> {
        let scope = self.scope;
        let spawned = self.spawned()?;
        let target = scope.target(spawned.leader());
        spawned.settled(deadline).inspect_err(|_| target.ended())
    }

    pub fn stdout(self, deadline: Instant) -> Result<String, RunFailure> {
        self.ran(deadline)?.succeeded()
    }

    pub fn detached(mut self) -> Result<(), RunFailure> {
        let line = CommandLine::from(&self.command);
        let quiet = KillScope::Group.led(&mut self.command).stdin(Stdio::null()).stdout(Stdio::null());
        quiet.stderr(Stdio::null()).spawn().map(drop).map_err(|source| RunFailure::Unspawned { line, source })
    }

    pub(crate) fn spawned(mut self) -> Result<Spawned, RunFailure> {
        let line = CommandLine::from(&self.command);
        let stdin = self.input.as_ref().map_or_else(Stdio::null, |_| Stdio::piped());
        let piped = self.scope.led(&mut self.command).stdin(stdin).stdout(Stdio::piped()).stderr(Stdio::piped());
        match piped.spawn() {
            Ok(child) => Ok(Spawned::new(line, child, self.input)),
            Err(source) => Err(RunFailure::Unspawned { line, source }),
        }
    }
}

impl From<&Run> for CommandLine {
    fn from(run: &Run) -> Self {
        Self::from(&run.command)
    }
}

impl From<&Run> for Environment {
    fn from(run: &Run) -> Self {
        Self::from(&run.command)
    }
}
