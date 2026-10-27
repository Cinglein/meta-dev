use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tracing::Span;
use tracing::field::{self, Empty};
use tracing::span::EnteredSpan;

use crate::span_field::SpanField;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanKind {
    Command,
    Spawn,
    Fs,
}

impl SpanKind {
    const COMMAND: &'static str = "command";
    const SPAWN: &'static str = "spawn";
    const FS: &'static str = "fs";

    pub fn opened(self) -> Span {
        match self {
            Self::Command => tracing::info_span!("command", label = Empty),
            Self::Spawn => tracing::info_span!("spawn", program = Empty, args = Empty, network = Empty, exit = Empty),
            Self::Fs => tracing::info_span!("fs", label = Empty),
        }
    }

    pub fn entered(self, label: &dyn fmt::Display) -> EnteredSpan {
        let span = self.opened();
        span.record(SpanField::Label.name(), field::display(label));
        span.entered()
    }
}

impl FromStr for SpanKind {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, ()> {
        match name {
            Self::COMMAND => Ok(Self::Command),
            Self::SPAWN => Ok(Self::Spawn),
            Self::FS => Ok(Self::Fs),
            _ => Err(()),
        }
    }
}

impl fmt::Display for SpanKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Command => Self::COMMAND,
            Self::Spawn => Self::SPAWN,
            Self::Fs => Self::FS,
        })
    }
}
