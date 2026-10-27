mod clock_reading;

use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::request::project::CiSettings;
use clock_reading::ClockReading;

#[derive(Debug)]
pub struct WatchClock {
    started: Instant,
    deadline: Instant,
    injected: Option<PathBuf>,
}

impl WatchClock {
    const INJECTED: &'static str = "META_TEST_CLOCK";
    const MARK: &'static str = " (injected clock)";

    pub fn start(settings: &CiSettings) -> Self {
        let started = Instant::now();
        let deadline = started + settings.remaining(Duration::ZERO).unwrap_or_default();
        Self { started, deadline, injected: std::env::var_os(Self::INJECTED).map(PathBuf::from) }
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    pub fn remaining(&self, settings: &CiSettings) -> Option<()> {
        self.deadline.checked_duration_since(Instant::now()).filter(|left| !left.is_zero())?;
        settings.remaining(self.elapsed()).map(drop)
    }

    pub fn elapsed(&self) -> Duration {
        let text = self.injected.as_ref().and_then(|file| std::fs::read_to_string(file).ok());
        let reading = text.and_then(|text| text.parse().ok());
        reading.map_or_else(|| self.started.elapsed(), ClockReading::elapsed)
    }
}

impl fmt::Display for WatchClock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.injected.as_ref().map_or("", |_| Self::MARK))
    }
}
