use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

use crate::dropped::Dropped;
use crate::kind_code::KindCode;
use crate::lane::Lane;
use crate::micros::Micros;
use crate::span_id::SpanId;
use crate::span_kind::SpanKind;
use crate::span_record::SpanRecord;
use crate::trace::Trace;

thread_local! {
    static CURRENT: Cell<Option<SpanId>> = const { Cell::new(None) };
}

#[derive(Debug)]
pub struct Recorder {
    spans: Mutex<Vec<SpanRecord>>,
    dropped: AtomicU64,
    first: SpanId,
    parent: Option<SpanId>,
}

impl Recorder {
    pub(crate) const CAPACITY: usize = 4096;

    pub fn new(parent: Option<SpanId>) -> Self {
        let spans = Mutex::new(Vec::with_capacity(Self::CAPACITY));
        Self { spans, dropped: AtomicU64::default(), first: SpanId::new(std::process::id(), 0), parent }
    }

    pub fn current() -> Option<SpanId> {
        CURRENT.get()
    }

    pub fn taken(&self) -> Trace {
        let dropped = Dropped::from(self.dropped.swap(0, Ordering::Relaxed));
        Trace::new(std::mem::take(&mut *self.locked())).short_by(dropped)
    }

    fn locked(&self) -> MutexGuard<'_, Vec<SpanRecord>> {
        self.spans.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Subscriber for Recorder {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, attributes: &Attributes<'_>) -> Id {
        let mut code = KindCode::UNKNOWN;
        attributes.record(&mut code);
        let mut spans = self.locked();
        let parent = CURRENT.get().or_else(|| spans.first().map(SpanRecord::span_id)).or(self.parent);
        let kind = SpanKind::try_from(code).ok().filter(|_| spans.len() < Self::CAPACITY);
        let id = self.first.indexed(kind.map_or(Self::CAPACITY, |_| spans.len()));
        self.dropped.fetch_add(u64::from(kind.is_none()), Ordering::Relaxed);
        spans.extend(kind.map(|kind| SpanRecord::new(id, parent, kind, Micros::now()).on_lane(Lane::current())));
        id.into()
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, _: &Event<'_>) {}

    fn enter(&self, span: &Id) {
        let span = SpanId::from(span);
        CURRENT.set(Some(span).filter(|span| span.index() < Self::CAPACITY).or(CURRENT.get()));
    }

    fn exit(&self, span: &Id) {
        let span = SpanId::from(span);
        let mut spans = self.locked();
        let record = spans.get_mut(span.index()).filter(|record| record.span_id() == span);
        let parent = record.map(|record| {
            *record = record.ended(Micros::now());
            record.parent()
        });
        CURRENT.set(parent.unwrap_or(CURRENT.get()));
    }
}
