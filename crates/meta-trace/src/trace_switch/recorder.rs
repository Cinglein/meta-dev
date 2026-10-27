mod chrome_layer;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt as _;

use crate::chrome_event::ChromeEvent;
use crate::trace::Trace;
use chrome_layer::ChromeLayer;

#[derive(Debug, Default)]
pub struct Recorder {
    events: Arc<Mutex<Vec<ChromeEvent>>>,
}

impl Recorder {
    pub fn recorded<Answer>(self, work: impl FnOnce() -> Answer, keep: impl FnOnce(Trace)) -> Answer {
        let layer = ChromeLayer::new(Arc::clone(&self.events), Instant::now());
        let answer = tracing::subscriber::with_default(Registry::default().with(layer), work);
        keep(Trace::new(self.events.lock().map(|mut events| std::mem::take(&mut *events)).unwrap_or_default()));
        answer
    }
}
