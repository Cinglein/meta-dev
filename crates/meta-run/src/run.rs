mod tree;

use std::ffi::OsStr;
use std::io::{Read, Write as _};
use std::ops::ControlFlow;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use crate::command_line::CommandLine;
use crate::env_policy::EnvPolicy;
use crate::environment::Environment;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use crate::stderr::Stderr;
use crate::stdout::Stdout;
use tree::Tree;

#[derive(Debug)]
pub struct Run {
    command: Command,
    ended: fn(&mut Child),
    input: Option<String>,
}

impl Run {
    const GIT: &'static str = "git";
    const DIRECTORY: &'static str = "-C";
    const NO_PROMPT: [&'static str; 2] = ["GIT_TERMINAL_PROMPT", "0"];
    const BATCH_SSH: [&'static str; 2] = ["GIT_SSH_COMMAND", "ssh -o BatchMode=yes"];
    const FIRST_PAUSE: Duration = Duration::from_micros(10);
    const LONGEST_PAUSE: Duration = Duration::from_millis(5);

    #[must_use]
    pub fn new(program: impl AsRef<OsStr>, env: EnvPolicy) -> Self {
        Self::inheriting(program, env, Environment::current())
    }

    #[must_use]
    pub fn inheriting(program: impl AsRef<OsStr>, env: EnvPolicy, inherited: Environment) -> Self {
        let mut command = Command::new(program);
        let kept = env.applied(inherited.clone());
        inherited.dropped(&kept, &mut command);
        Self { command, ended: Tree::ended, input: None }
    }

    #[must_use]
    pub fn git(dir: &Path) -> Self {
        let git = Self::new(Self::GIT, EnvPolicy::NoRepoVars).var(Self::NO_PROMPT[0], Self::NO_PROMPT[1]);
        let ssh = std::env::var_os(Self::BATCH_SSH[0]).is_none().then_some(Self::BATCH_SSH);
        let git = ssh.into_iter().fold(git, |git, [name, value]| git.var(name, value));
        git.arg(Self::DIRECTORY).arg(dir)
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

    pub fn ran(mut self, deadline: Instant) -> Result<Ran, RunFailure> {
        let line = CommandLine::from(&self.command);
        let stdin = self.input.as_ref().map_or_else(Stdio::null, |_| Stdio::piped());
        let piped = self.command.stdin(stdin).stdout(Stdio::piped()).stderr(Stdio::piped());
        let spawned = piped.spawn().map_err(|source| RunFailure::Unspawned { line: line.clone(), source });
        let mut child = spawned?;
        let fed = child.stdin.take().zip(self.input);
        fed.map(|(mut pipe, input)| std::thread::spawn(move || pipe.write_all(input.as_bytes())));
        let stdout = Self::drained(child.stdout.take(), Stdout::read_from);
        let stderr = Self::drained(child.stderr.take(), Stderr::read_from);
        let left = || deadline.saturating_duration_since(Instant::now());
        let streams = stdout.recv_timeout(left()).ok().zip(stderr.recv_timeout(left()).ok());
        let status = streams.as_ref().and_then(|_| Self::exited(&mut child, deadline));
        let ran = status.zip(streams).map(|(status, (stdout, stderr))| Ran::new(line.clone(), status, stdout, stderr));
        ran.ok_or_else(|| {
            (self.ended)(&mut child);
            RunFailure::TimedOut(line)
        })
    }

    pub fn stdout(self, deadline: Instant) -> Result<String, RunFailure> {
        self.ran(deadline)?.succeeded()
    }

    pub fn detached(mut self) -> Result<(), RunFailure> {
        let line = CommandLine::from(&self.command);
        let quiet = self.command.process_group(i32::default()).stdin(Stdio::null()).stdout(Stdio::null());
        quiet.stderr(Stdio::null()).spawn().map(drop).map_err(|source| RunFailure::Unspawned { line, source })
    }

    fn alone(child: &mut Child) {
        drop(child.kill());
        drop(child.wait());
    }

    fn drained<Pipe: Read + Send + 'static, Text: Send + 'static>(
        pipe: Option<Pipe>,
        read: fn(Option<Pipe>) -> Text,
    ) -> Receiver<Text> {
        let (sender, drained) = channel();
        std::thread::spawn(move || sender.send(read(pipe)));
        drained
    }

    fn exited(child: &mut Child, deadline: Instant) -> Option<ExitStatus> {
        let pauses = std::iter::successors(Some(Self::FIRST_PAUSE), |pause| {
            Some(pause.saturating_add(*pause).min(Self::LONGEST_PAUSE))
        });
        pauses.into_iter().try_for_each(|pause| Self::waited(child, pause, deadline)).break_value().flatten()
    }

    fn waited(child: &mut Child, pause: Duration, deadline: Instant) -> ControlFlow<Option<ExitStatus>> {
        let left = deadline.checked_duration_since(Instant::now());
        match (child.try_wait(), left) {
            (Ok(None), Some(left)) => {
                std::thread::sleep(pause.min(left));
                ControlFlow::Continue(())
            }
            (Ok(status), _) => ControlFlow::Break(status),
            (Err(_), _) => ControlFlow::Break(None),
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
