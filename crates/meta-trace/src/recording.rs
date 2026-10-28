use std::io;
use std::sync::Arc;

use tracing::span::EnteredSpan;

use crate::recorder::Recorder;
use crate::retention::Retention;
use crate::span_kind::SpanKind;
use crate::stitch::Stitch;
use crate::trace::Trace;
use crate::trace_dir::TraceDir;
use crate::trace_id::TraceId;
use crate::verb::Verb;

#[derive(Debug)]
pub struct Recording {
    recorder: Arc<Recorder>,
    trace: TraceId,
    command: EnteredSpan,
}

impl Recording {
    #[must_use]
    pub fn started(verb: Verb) -> Option<Self> {
        let stitch = std::env::var(Stitch::VARIABLE).ok().and_then(|stitch| stitch.parse::<Stitch>().ok());
        let trace = stitch.map_or_else(TraceId::new, Stitch::trace);
        let recorder = Arc::new(Recorder::new(stitch.map(Stitch::parent)));
        tracing::subscriber::set_global_default(Arc::clone(&recorder)).ok()?;
        Stitch::installed(trace).then(|| Self { recorder, trace, command: SpanKind::Command(verb).entered() })
    }

    pub fn kept(self, dir: &TraceDir, retention: Retention) -> io::Result<()> {
        drop(self.command);
        dir.kept(self.trace, &Trace::new(self.recorder.taken()), retention)
    }
}
