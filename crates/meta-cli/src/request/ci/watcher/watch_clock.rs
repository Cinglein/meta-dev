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
    origin: Duration,
}

impl WatchClock {
    const INJECTED: &'static str = "META_TEST_CLOCK";
    const MARK: &'static str = " (injected clock)";

    pub fn start(settings: &CiSettings) -> Self {
        let started = Instant::now();
        let deadline = started + settings.remaining(Duration::ZERO).unwrap_or_default();
        let injected = std::env::var_os(Self::INJECTED).map(PathBuf::from);
        let origin = Self::reading(injected.as_ref()).map_or(Duration::ZERO, ClockReading::elapsed);
        Self { started, deadline, injected, origin }
    }

    pub fn mark() -> &'static str {
        std::env::var_os(Self::INJECTED).map_or("", |_| Self::MARK)
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    pub fn remaining(&self, settings: &CiSettings) -> Option<()> {
        self.deadline.checked_duration_since(Instant::now()).filter(|left| !left.is_zero())?;
        settings.remaining(self.elapsed()).map(drop)
    }

    pub fn elapsed(&self) -> Duration {
        Self::reading(self.injected.as_ref())
            .map_or_else(|| self.started.elapsed(), |now| now.elapsed().saturating_sub(self.origin))
    }

    fn reading(injected: Option<&PathBuf>) -> Option<ClockReading> {
        injected.and_then(|file| std::fs::read_to_string(file).ok()).and_then(|text| text.parse().ok())
    }
}

impl fmt::Display for WatchClock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.injected.as_ref().map_or("", |_| Self::MARK))
    }
}
