mod group;
mod interrupts;
mod placement;
#[cfg(test)]
mod tests;

use std::ffi::{OsStr, OsString};
use std::io::{Read, Write as _};
use std::ops::ControlFlow;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use meta_trace::{SpanKind, Stitch};

use crate::Reported;
use crate::command_line::CommandLine;
use crate::env_policy::EnvPolicy;
use crate::environment::Environment;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use crate::spawn_trace::SpawnTrace;
use crate::stderr::Stderr;
use crate::stdout::Stdout;
use crate::unread::Unread;
use group::Group;
use interrupts::Interrupts;
use placement::Placement;

#[derive(Debug)]
pub struct Run {
    command: Command,
    input: Option<String>,
    placement: Placement,
}

impl Run {
    const GIT: &'static str = "git";
    const DIRECTORY: &'static str = "-C";
    pub(crate) const NO_PROMPT: [&'static str; 2] = ["GIT_TERMINAL_PROMPT", "0"];
    const FIRST_PAUSE: Duration = Duration::from_micros(10);
    const LONGEST_PAUSE: Duration = Duration::from_millis(5);

    #[must_use]
    pub fn new(program: impl AsRef<OsStr>, env: EnvPolicy) -> Self {
        Self::inheriting(program, env, Environment::current())
    }

    #[must_use]
    pub fn inheriting(program: impl AsRef<OsStr>, env: EnvPolicy, inherited: Environment) -> Self {
        let run = Self { command: Command::new(program), input: None, placement: Placement::Own };
        run.policed_from(env, inherited)
    }

    #[must_use]
    pub fn policed(self, env: EnvPolicy) -> Self {
        self.policed_from(env, Environment::current())
    }

    #[must_use]
    pub fn policed_from(mut self, env: EnvPolicy, inherited: Environment) -> Self {
        let kept = env.applied(inherited.clone());
        inherited.dropped(&kept, &mut self.command);
        self
    }

    #[must_use]
    pub fn git(dir: &Path) -> Self {
        let git = Self::new(Self::GIT, EnvPolicy::NoRepoVars).var(Self::NO_PROMPT[0], Self::NO_PROMPT[1]);
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

    #[must_use]
    pub fn nested(program: impl AsRef<OsStr>) -> Self {
        let run = Self::new(program, EnvPolicy::Inherit);
        Self { command: run.command, input: run.input, placement: Placement::Nested }
    }

    #[must_use]
    pub fn terminal(mut self) -> Self {
        let prompt = Self::NO_PROMPT[0];
        self.command.env_remove(prompt);
        let terminal = std::env::var_os(prompt).into_iter().fold(self, |run, inherited| run.var(prompt, inherited));
        Self { command: terminal.command, input: terminal.input, placement: Placement::Terminal }
    }

    pub fn ran(self, deadline: Instant) -> Result<Ran, RunFailure> {
        let kind = SpanKind::spawning(&self.command);
        let _span = kind.entered();
        let run = kind.stitch().into_iter().fold(self, |run, stitch| run.var(Stitch::VARIABLE, OsString::from(stitch)));
        let watch = Interrupts::watched();
        let line = CommandLine::from(&run.command);
        let ran =
            Group::accepting().ok_or(RunFailure::Interrupted(line)).and_then(|()| run.placement.ran(run, deadline));
        watch.honored();
        ran
    }

    #[must_use]
    pub fn exit(code: ExitCode) -> ExitCode {
        Interrupts::watched().honored();
        code
    }

    pub(crate) fn helped(self, within: Duration) -> Result<Ran, RunFailure> {
        self.finished(Instant::now() + within, |_| Some(()), Self::alone)
    }

    pub(crate) fn alone(child: &mut Child) {
        drop(child.kill());
        drop(child.wait());
    }

    fn finished<Member>(
        mut self,
        deadline: Instant,
        joined: impl FnOnce(&mut Child) -> Option<Member>,
        ended: impl FnOnce(&mut Child),
    ) -> Result<Ran, RunFailure> {
        let line = CommandLine::from(&self.command);
        deadline.checked_duration_since(Instant::now()).ok_or_else(|| RunFailure::Expired(line.clone()))?;
        let stdin = self.input.as_ref().map_or_else(Stdio::null, |_| Stdio::piped());
        let piped = self.command.stdin(stdin).stdout(Stdio::piped()).stderr(Stdio::piped());
        let unspawned = |source| RunFailure::Unspawned { line: line.clone(), source };
        SpawnTrace::started(piped).map_err(unspawned)?;
        let mut child = piped.spawn().map_err(unspawned)?;
        let _member = joined(&mut child).ok_or_else(|| RunFailure::Interrupted(line.clone()))?;
        let fed = child.stdin.take().zip(self.input);
        fed.map(|(mut pipe, input)| std::thread::spawn(move || pipe.write_all(input.as_bytes())));
        let stdout = Self::drained(child.stdout.take(), Stdout::read_from);
        let stderr = Self::drained(child.stderr.take(), Stderr::read_from);
        let left = || deadline.saturating_duration_since(Instant::now());
        let streams = stdout.recv_timeout(left()).ok().zip(stderr.recv_timeout(left()).ok());
        let status = streams.as_ref().and_then(|_| Self::exited(&mut child, deadline));
        let ran = status.zip(streams).map(|(status, (stdout, stderr))| Ran::new(line.clone(), status, stdout, stderr));
        ran.ok_or_else(|| {
            ended(&mut child);
            RunFailure::TimedOut(line)
        })
    }

    pub fn stdout(self, deadline: Instant) -> Result<String, RunFailure> {
        self.ran(deadline)?.succeeded()
    }

    pub fn reported(self, deadline: Instant) -> Reported {
        let ran = self.ran(deadline)?;
        if ran.success() { Ok(ran.into_stdout()) } else { Err(Unread::Failed(ran.status(), ran.stderr().clone())) }
    }

    pub fn detached(mut self) -> Result<(), RunFailure> {
        let line = CommandLine::from(&self.command);
        let _watched = Interrupts::watched();
        Group::accepting().ok_or_else(|| RunFailure::Interrupted(line.clone()))?;
        let quiet = self.command.process_group(i32::default()).stdin(Stdio::null()).stdout(Stdio::null());
        let unspawned = |source| RunFailure::Unspawned { line: line.clone(), source };
        SpawnTrace::started(quiet).map_err(unspawned)?;
        quiet.stderr(Stdio::null()).spawn().map(drop).map_err(unspawned)
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
