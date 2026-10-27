mod clock_reading;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use clock_reading::ClockReading;

#[derive(Debug)]
pub struct WatchClock {
    started: Instant,
    injected: Option<PathBuf>,
}

impl WatchClock {
    const INJECTED: &'static str = "META_CI_CLOCK";

    pub fn start() -> Self {
        Self { started: Instant::now(), injected: std::env::var_os(Self::INJECTED).map(PathBuf::from) }
    }

    pub fn started(&self) -> Instant {
        self.started
    }

    pub fn elapsed(&self) -> Duration {
        let text = self.injected.as_ref().and_then(|file| std::fs::read_to_string(file).ok());
        let reading = text.and_then(|text| text.parse().ok());
        reading.map_or_else(|| self.started.elapsed(), ClockReading::elapsed)
    }
}
