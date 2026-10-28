use std::io::{self, Read, Write as _};
use std::process::{Child, ExitStatus};
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

use crate::command_line::CommandLine;
use crate::leader::Leader;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use crate::stderr::Stderr;
use crate::stdout::Stdout;

#[derive(Debug)]
pub struct Spawned {
    line: CommandLine,
    leader: Leader,
    exited: Receiver<io::Result<ExitStatus>>,
    stdout: Receiver<Stdout>,
    stderr: Receiver<Stderr>,
}

impl Spawned {
    pub fn new(line: CommandLine, mut child: Child, input: Option<String>) -> Self {
        let fed = child.stdin.take().zip(input);
        fed.map(|(mut pipe, input)| std::thread::spawn(move || pipe.write_all(input.as_bytes())));
        let stdout = Self::drained(child.stdout.take(), Stdout::read_from);
        let stderr = Self::drained(child.stderr.take(), Stderr::read_from);
        let leader = Leader::from(&child);
        let (sender, exited) = channel();
        std::thread::spawn(move || sender.send(child.wait()));
        Self { line, leader, exited, stdout, stderr }
    }

    pub fn leader(&self) -> Leader {
        self.leader
    }

    pub fn settled(self, deadline: Instant) -> Result<Ran, RunFailure> {
        let left = || deadline.saturating_duration_since(Instant::now());
        let timed_out = || RunFailure::TimedOut(self.line.clone());
        let status = self.exited.recv_timeout(left()).map_err(|_| timed_out())?;
        let status = status.map_err(|source| RunFailure::Unspawned { line: self.line.clone(), source })?;
        let stdout = self.stdout.recv_timeout(left()).map_err(|_| timed_out())?;
        let stderr = self.stderr.recv_timeout(left()).map_err(|_| timed_out())?;
        Ok(Ran::new(self.line, status, stdout, stderr))
    }

    fn drained<Pipe: Read + Send + 'static, Text: Send + 'static>(
        pipe: Option<Pipe>,
        read: fn(Option<Pipe>) -> Text,
    ) -> Receiver<Text> {
        let (sender, drained) = channel();
        std::thread::spawn(move || sender.send(read(pipe)));
        drained
    }
}
