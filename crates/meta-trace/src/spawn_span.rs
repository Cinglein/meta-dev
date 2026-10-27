mod spawned_words;

use std::fmt;
use std::io;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output};

use tracing::field;
use tracing::span::EnteredSpan;

use crate::span_field::SpanField;
use crate::span_kind::SpanKind;
use spawned_words::SpawnedWords;

#[derive(Debug)]
pub struct SpawnSpan(EnteredSpan);

impl SpawnSpan {
    const UNSPAWNED: &'static str = "unspawned";
    const DETACHED: &'static str = "detached";

    #[must_use]
    pub fn opened(command: &Command) -> Self {
        let span = SpanKind::Spawn.opened();
        let words = SpawnedWords::new(command);
        let program = Path::new(command.get_program()).file_name().unwrap_or_default();
        span.record(SpanField::Program.name(), field::display(program.display()));
        span.record(SpanField::Args.name(), field::display(words));
        span.record(SpanField::Network.name(), words.networked());
        Self(span.entered())
    }

    pub fn ended(self, status: Option<&ExitStatus>) {
        let exit: &dyn fmt::Display = match status {
            Some(status) => status,
            None => &Self::UNSPAWNED,
        };
        self.0.record(SpanField::Exit.name(), field::display(exit));
    }

    pub fn output_of(command: &mut Command) -> io::Result<Output> {
        let span = Self::opened(command);
        let output = command.output();
        span.ended(output.as_ref().ok().map(|output| &output.status));
        output
    }

    pub fn status_of(command: &mut Command) -> io::Result<ExitStatus> {
        let span = Self::opened(command);
        let status = command.status();
        span.ended(status.as_ref().ok());
        status
    }

    pub fn detached(command: &mut Command) -> io::Result<Child> {
        let span = Self::opened(command);
        let child = command.spawn();
        span.0.record(SpanField::Exit.name(), field::display(Self::DETACHED));
        child
    }
}
