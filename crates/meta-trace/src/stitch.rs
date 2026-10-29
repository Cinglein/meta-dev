use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

use crate::recorder::Recorder;
use crate::span_id::SpanId;
use crate::trace_id::TraceId;

static TRACE: OnceLock<TraceId> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stitch {
    trace: TraceId,
    parent: SpanId,
}

impl Stitch {
    pub const VARIABLE: &'static str = "META_TRACE_PARENT";
    const JOINER: char = '-';

    #[must_use]
    pub fn current() -> Option<Self> {
        TRACE.get().copied().zip(Recorder::current()).map(|(trace, parent)| Self { trace, parent })
    }

    pub(crate) fn installed(trace: TraceId) -> bool {
        TRACE.set(trace).is_ok()
    }

    pub(crate) fn trace(self) -> TraceId {
        self.trace
    }

    pub(crate) fn parent(self) -> SpanId {
        self.parent
    }
}

impl FromStr for Stitch {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let (trace, parent) = text.split_once(Self::JOINER).ok_or(())?;
        let trace = trace.parse().ok().filter(|trace| *trace <= TraceId::new()).ok_or(())?;
        Ok(Self { trace, parent: parent.parse()? })
    }
}

impl fmt::Display for Stitch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}{}", self.trace, Self::JOINER, self.parent)
    }
}

impl From<Stitch> for OsString {
    fn from(stitch: Stitch) -> Self {
        stitch.to_string().into()
    }
}
