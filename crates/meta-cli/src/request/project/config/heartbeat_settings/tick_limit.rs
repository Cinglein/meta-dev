use std::fmt;
use std::sync::mpsc;
use std::time::Duration;

use serde::Deserialize;

use super::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Seconds")]
pub struct TickLimit(Duration);

impl TickLimit {
    pub fn within<Done: Send + 'static>(self, work: impl FnOnce() -> Done + Send + 'static) -> Option<Done> {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || sender.send(work()).ok());
        receiver.recv_timeout(self.0).ok()
    }

    pub fn under(self, interval: Seconds) -> Option<()> {
        (self.0 < interval.duration()).then_some(())
    }
}

impl From<Seconds> for TickLimit {
    fn from(seconds: Seconds) -> Self {
        Self(seconds.duration())
    }
}

impl From<Duration> for TickLimit {
    fn from(limit: Duration) -> Self {
        Self(limit)
    }
}

impl fmt::Display for TickLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s", self.0.as_secs())
    }
}
