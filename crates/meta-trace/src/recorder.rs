use std::cell::Cell;
use std::sync::{Mutex, MutexGuard, PoisonError};

use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

use crate::kind_code::KindCode;
use crate::micros::Micros;
use crate::span_id::SpanId;
use crate::span_kind::SpanKind;
use crate::span_record::SpanRecord;

thread_local! {
    static CURRENT: Cell<Option<SpanId>> = const { Cell::new(None) };
}

#[derive(Debug)]
pub struct Recorder {
    spans: Mutex<Vec<SpanRecord>>,
    first: SpanId,
    parent: Option<SpanId>,
}

impl Recorder {
    const CAPACITY: usize = 4096;

    pub fn new(parent: Option<SpanId>) -> Self {
        let spans = Mutex::new(Vec::with_capacity(Self::CAPACITY));
        Self { spans, first: SpanId::new(std::process::id(), 0), parent }
    }

    pub fn current() -> Option<SpanId> {
        CURRENT.get()
    }

    pub fn taken(&self) -> Vec<SpanRecord> {
        std::mem::take(&mut *self.locked())
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
        let id = self.first.indexed(spans.len());
        let parent = CURRENT.get().or_else(|| spans.first().map(SpanRecord::span_id)).or(self.parent);
        let kind = SpanKind::try_from(code).ok().filter(|_| spans.len() != Self::CAPACITY);
        spans.extend(kind.map(|kind| SpanRecord::new(id, parent, kind, Micros::now())));
        id.into()
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, _: &Event<'_>) {}

    fn enter(&self, span: &Id) {
        CURRENT.set(Some(SpanId::from(span)));
    }

    fn exit(&self, span: &Id) {
        let span = SpanId::from(span);
        let mut spans = self.locked();
        let record = spans.get_mut(span.index()).filter(|record| record.span_id() == span);
        let parent = record.and_then(|record| {
            *record = record.ended(Micros::now());
            record.parent()
        });
        CURRENT.set(parent);
    }
}
