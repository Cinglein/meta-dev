mod kind_times;

use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::fmt;
use std::ops::Range;

use crate::dropped::Dropped;
use crate::micros::Micros;
use crate::span_id::SpanId;
use crate::span_kind::SpanKind;
use crate::span_record::SpanRecord;
use crate::verb::Verb;
use kind_times::KindTimes;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Trace {
    spans: Vec<SpanRecord>,
    dropped: Dropped,
}

impl Trace {
    const MAGIC: &'static str = "mtr2";

    #[must_use]
    pub fn new(spans: Vec<SpanRecord>) -> Self {
        Self { spans, dropped: Dropped::default() }
    }

    #[must_use]
    pub fn root(&self) -> Option<&SpanRecord> {
        let ids: BTreeSet<_> = self.spans.iter().map(SpanRecord::span_id).collect();
        let roots = self.spans.iter().filter(|span| span.parent().is_none_or(|parent| !ids.contains(&parent)));
        roots.min_by_key(|span| span.window().start)
    }

    #[must_use]
    pub fn started_by(&self, verb: Option<Verb>) -> bool {
        self.root().is_some_and(|root| verb.is_none_or(|verb| root.kind() == SpanKind::Command(verb)))
    }

    pub fn chrome(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.spans)
    }

    pub(crate) fn short_by(self, dropped: Dropped) -> Self {
        Self { dropped, ..self }
    }

    pub(crate) fn repeated(&self) -> Option<SpanId> {
        let mut seen = BTreeSet::new();
        self.spans.iter().map(SpanRecord::span_id).find(|span| !seen.insert(*span))
    }

    pub(crate) fn chained_children(&self, span: &SpanRecord) -> Vec<&SpanRecord> {
        let mut latest_first: Vec<&SpanRecord> = self.children(span).collect();
        latest_first.sort_by_key(|child| Reverse(child.window().end));
        let chosen = latest_first.into_iter().scan(span.window().end, |cursor, child| {
            let chosen = Some(child).filter(|child| child.window().end <= *cursor);
            *cursor = chosen.map_or(*cursor, |child| child.window().start);
            Some(chosen)
        });
        chosen.flatten().collect()
    }

    fn critical(&self, root: &SpanRecord) -> KindTimes {
        let mut pending = vec![root];
        let walked = std::iter::from_fn(|| {
            let span = pending.pop()?;
            let path = self.chained_children(span);
            let own = span.duration() - path.iter().map(|child| child.duration()).sum();
            pending.extend(path);
            Some((span.kind(), own))
        });
        walked.take(self.spans.len()).collect()
    }

    fn untraced(&self) -> KindTimes {
        let parents: BTreeSet<_> = self.spans.iter().filter_map(SpanRecord::parent).collect();
        let open = self
            .spans
            .iter()
            .filter(|span| parents.contains(&span.span_id()) || matches!(span.kind(), SpanKind::Command(_)));
        open.map(|span| (span.kind(), span.duration() - self.covered(span))).collect()
    }

    fn children<'trace>(&'trace self, parent: &SpanRecord) -> impl Iterator<Item = &'trace SpanRecord> {
        let id = parent.span_id();
        self.spans.iter().filter(move |span| span.parent() == Some(id))
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
        let head = Trace::MAGIC.as_bytes().iter().copied().chain(trace.dropped.bytes());
        head.chain(trace.spans.iter().flat_map(Vec::from)).collect()
    }
}

impl From<&[u8]> for Trace {
    fn from(bytes: &[u8]) -> Self {
        let body = bytes.strip_prefix(Self::MAGIC.as_bytes()).and_then(<[u8]>::split_first_chunk);
        let (dropped, records) = body.map(|(head, records)| (Dropped::from(*head), records)).unwrap_or_default();
        let spans = records.as_chunks().0.iter().filter_map(|record| SpanRecord::try_from(record).ok());
        Self { spans: spans.collect(), dropped }
    }
}

impl FromIterator<Trace> for Trace {
    fn from_iter<Traces: IntoIterator<Item = Trace>>(traces: Traces) -> Self {
        traces.into_iter().fold(Self::default(), |mut merged, trace| {
            merged.spans.extend(trace.spans);
            Self { dropped: merged.dropped + trace.dropped, ..merged }
        })
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.root() {
            None => formatter.write_str("empty trace\n"),
            Some(root) => {
                let processes: BTreeSet<u32> = self.spans.iter().map(|span| span.span_id().pid()).collect();
                let [processes, spans] = [processes.len(), self.spans.len()];
                writeln!(formatter, "{} {}, {processes} processes, {spans} spans", root.kind(), root.duration())?;
                let walked = || format!("critical path by kind\n{}untraced\n{}", self.critical(root), self.untraced());
                let shown =
                    self.repeated().map_or_else(walked, |span| format!("corrupt trace: span {span} recorded twice\n"));
                write!(formatter, "{}{shown}", self.dropped)
            }
        }
    }
}
