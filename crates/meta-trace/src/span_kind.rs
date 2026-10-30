use std::ffi::OsStr;
use std::fmt;

use tracing::span::EnteredSpan;

use crate::kind_code::KindCode;
use crate::kind_tag::KindTag;
use crate::program::Program;
use crate::stitch::Stitch;
use crate::verb::Verb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpanKind {
    Command(Verb),
    Spawn(Program),
    Network(Program),
    Fs,
}

impl SpanKind {
    #[must_use]
    pub fn spawning(program: &OsStr, args: &[&OsStr]) -> Self {
        Program::named(program).spawned(args)
    }

    pub fn entered(self) -> EnteredSpan {
        tracing::info_span!("meta", kind = u64::from(KindCode::from(self).word())).entered()
    }

    #[must_use]
    pub fn stitch(self) -> Option<Stitch> {
        (self == Self::Spawn(Program::Meta)).then(Stitch::current).flatten()
    }
}

impl From<SpanKind> for KindCode {
    fn from(kind: SpanKind) -> Self {
        match kind {
            SpanKind::Command(verb) => Self::new(KindTag::Command, &verb),
            SpanKind::Spawn(program) => Self::new(KindTag::Spawn, &program),
            SpanKind::Network(program) => Self::new(KindTag::Network, &program),
            SpanKind::Fs => Self::new(KindTag::Fs, &KindTag::Command),
        }
    }
}

impl TryFrom<KindCode> for SpanKind {
    type Error = ();

    fn try_from(code: KindCode) -> Result<Self, ()> {
        code.tag().and_then(|tag| tag.kind(code)).ok_or(())
    }
}

impl fmt::Display for SpanKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(verb) => write!(formatter, "command {verb}"),
            Self::Spawn(program) => write!(formatter, "spawn {program}"),
            Self::Network(program) => write!(formatter, "network {program}"),
            Self::Fs => formatter.write_str("fs"),
        }
    }
}
