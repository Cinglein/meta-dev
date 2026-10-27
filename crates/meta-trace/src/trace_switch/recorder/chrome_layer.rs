use std::sync::{Arc, Mutex};
use std::time::Instant;

use tracing::Subscriber;
use tracing::span::{Attributes, Id, Record};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use crate::chrome_event::ChromeEvent;
use crate::micros::Micros;
use crate::span_args::SpanArgs;
use crate::span_kind::SpanKind;

#[derive(Debug)]
pub struct ChromeLayer {
    events: Arc<Mutex<Vec<ChromeEvent>>>,
    origin: Instant,
}

impl ChromeLayer {
    pub fn new(events: Arc<Mutex<Vec<ChromeEvent>>>, origin: Instant) -> Self {
        Self { events, origin }
    }
}

impl<Inner> Layer<Inner> for ChromeLayer
where
    Inner: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, Inner>) {
        let mut args = SpanArgs::default();
        attributes.record(&mut args);
        let kind = attributes.metadata().name().parse::<SpanKind>().ok();
        let opened = kind.map(|kind| ChromeEvent::opened(kind, args, Micros::elapsed_since(self.origin)));
        context.span(id).zip(opened).into_iter().for_each(|(span, event)| span.extensions_mut().insert(event));
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, Inner>) {
        context.span(id).into_iter().for_each(|span| {
            span.extensions_mut().get_mut::<ChromeEvent>().into_iter().for_each(|event| event.recorded(values));
        });
    }

    fn on_close(&self, id: Id, context: Context<'_, Inner>) {
        let open = context.span(&id).and_then(|span| span.extensions_mut().remove::<ChromeEvent>());
        let end = Micros::elapsed_since(self.origin);
        open.zip(self.events.lock().ok()).into_iter().for_each(|(event, mut events)| events.push(event.closed(end)));
    }
}
