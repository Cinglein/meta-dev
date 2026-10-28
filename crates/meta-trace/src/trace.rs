mod kind_times;

use std::collections::BTreeSet;
use std::fmt;
use std::ops::Range;

use crate::micros::Micros;
use crate::span_kind::SpanKind;
use crate::span_record::SpanRecord;
use crate::verb::Verb;
use kind_times::KindTimes;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Trace(Vec<SpanRecord>);

impl Trace {
    const MAGIC: &'static str = "mtr1";

    #[must_use]
    pub fn new(spans: Vec<SpanRecord>) -> Self {
        Self(spans)
    }

    #[must_use]
    pub fn root(&self) -> Option<&SpanRecord> {
        let ids: BTreeSet<_> = self.0.iter().map(SpanRecord::span_id).collect();
        let roots = self.0.iter().filter(|span| span.parent().is_none_or(|parent| !ids.contains(&parent)));
        roots.min_by_key(|span| span.window().start)
    }

    #[must_use]
    pub fn started_by(&self, verb: Option<Verb>) -> bool {
        self.root().is_some_and(|root| verb.is_none_or(|verb| root.kind() == SpanKind::Command(verb)))
    }

    pub fn chrome(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.0)
    }

    fn critical(&self, span: &SpanRecord) -> KindTimes {
        let latest = |cursor: Micros| {
            let before =
                self.children(span).filter(|child| child.window().end <= cursor && child.window().start < cursor);
            before.max_by_key(|child| child.window().end)
        };
        let path: Vec<&SpanRecord> =
            std::iter::successors(latest(span.window().end), |child| latest(child.window().start)).collect();
        let own = span.duration() - path.iter().map(|child| child.duration()).sum();
        std::iter::once((span.kind(), own)).chain(path.iter().flat_map(|child| self.critical(child))).collect()
    }

    fn untraced(&self) -> KindTimes {
        let parents: BTreeSet<_> = self.0.iter().filter_map(SpanRecord::parent).collect();
        let open = self
            .0
            .iter()
            .filter(|span| parents.contains(&span.span_id()) || matches!(span.kind(), SpanKind::Command(_)));
        open.map(|span| (span.kind(), span.duration() - self.covered(span))).collect()
    }

    fn children<'trace>(&'trace self, parent: &SpanRecord) -> impl Iterator<Item = &'trace SpanRecord> {
        let id = parent.span_id();
        self.0.iter().filter(move |span| span.parent() == Some(id))
    }

    fn covered(&self, span: &SpanRecord) -> Micros {
        let mut windows: Vec<Range<Micros>> = self.children(span).map(SpanRecord::window).collect();
        windows.sort_by_key(|window| window.start);
        let bounds = span.window();
        let parts = windows.iter().scan(bounds.start, |reach, window| {
            let begin = window.start.max(*reach);
            let finish = window.end.min(bounds.end).max(begin);
            *reach = finish.max(*reach);
            Some(finish - begin)
        });
        parts.sum()
    }
}

impl From<&Trace> for Vec<u8> {
    fn from(trace: &Trace) -> Self {
        Trace::MAGIC.as_bytes().iter().copied().chain(trace.0.iter().flat_map(Vec::from)).collect()
    }
}

impl From<&[u8]> for Trace {
    fn from(bytes: &[u8]) -> Self {
        let records = bytes.strip_prefix(Self::MAGIC.as_bytes()).map(|records| records.as_chunks().0);
        Self(records.unwrap_or_default().iter().filter_map(|record| SpanRecord::try_from(record).ok()).collect())
    }
}

impl FromIterator<Trace> for Trace {
    fn from_iter<Traces: IntoIterator<Item = Trace>>(traces: Traces) -> Self {
        Self(traces.into_iter().flat_map(|trace| trace.0).collect())
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.root() {
            None => formatter.write_str("empty trace\n"),
            Some(root) => {
                let processes: BTreeSet<u32> = self.0.iter().map(|span| span.span_id().pid()).collect();
                let [processes, spans] = [processes.len(), self.0.len()];
                writeln!(formatter, "{} {}, {processes} processes, {spans} spans", root.kind(), root.duration())?;
                write!(formatter, "critical path by kind\n{}untraced\n{}", self.critical(root), self.untraced())
            }
        }
    }
}
