mod elapsed;
mod phase;
mod process_id;

use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::span::Record;

use crate::field_text::FieldText;
use crate::micros::Micros;
use crate::placement::Placement;
use crate::span_args::SpanArgs;
use crate::span_kind::SpanKind;
use elapsed::Elapsed;
use phase::Phase;
use process_id::ProcessId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChromeEvent {
    name: FieldText,
    #[serde(rename = "cat")]
    kind: SpanKind,
    #[serde(rename = "ph")]
    phase: Phase,
    #[serde(rename = "ts")]
    start: Micros,
    #[serde(rename = "dur")]
    duration: Elapsed,
    pid: ProcessId,
    #[serde(rename = "tid")]
    thread: ProcessId,
    #[serde(default)]
    args: SpanArgs,
}

impl ChromeEvent {
    pub fn opened(kind: SpanKind, args: SpanArgs, start: Micros) -> Self {
        let pid = ProcessId::own();
        let name = FieldText::default();
        Self { name, kind, phase: Phase::Complete, start, duration: Elapsed::default(), pid, thread: pid, args }
    }

    pub fn recorded(&mut self, values: &Record<'_>) {
        values.record(&mut self.args);
    }

    #[must_use]
    pub fn closed(self, end: Micros) -> Self {
        Self { name: FieldText::titled(&self.args), duration: Elapsed::between(self.start, end), ..self }
    }

    pub fn duration(&self) -> Elapsed {
        self.duration
    }

    pub fn end(&self) -> Micros {
        self.start + self.duration
    }

    pub fn placing(&self, other: &Self) -> Placement {
        if other.duration < self.duration && self.start <= other.start && other.end() <= self.end() {
            Placement::Inside
        } else if other.end() <= self.start && other.start < self.start {
            Placement::Before
        } else {
            Placement::Elsewhere
        }
    }

    pub fn spawned(&self) -> Option<&FieldText> {
        (self.kind == SpanKind::Spawn).then(|| self.args.program()).flatten()
    }

    pub fn args(&self) -> &SpanArgs {
        &self.args
    }
}

impl fmt::Display for ChromeEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:>10} {} {}", self.duration, self.kind, self.name)
    }
}
