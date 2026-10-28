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
        self.left(settings, Duration::ZERO).map(drop)
    }

    pub fn pause(&self, wait: Duration, settings: &CiSettings) -> Option<()> {
        let pause = self.left(settings, settings.gh_limit()).map(|window| wait.min(window))?;
        let reading = Self::reading(self.injected.as_ref()).map_or(self.origin, ClockReading::elapsed);
        let advanced =
            self.injected.as_ref().map(|file| meta_door::Door::write(file, (reading + pause).as_millis().to_string()));
        advanced.map_or_else(|| std::thread::sleep(pause), drop);
        Some(())
    }

    pub fn elapsed(&self) -> Duration {
        Self::reading(self.injected.as_ref())
            .map_or_else(|| self.started.elapsed(), |now| now.elapsed().saturating_sub(self.origin))
    }

    fn left(&self, settings: &CiSettings, reserve: Duration) -> Option<Duration> {
        let beyond = |left: Duration| left.checked_sub(reserve).filter(|rest| !rest.is_zero());
        beyond(self.deadline.checked_duration_since(Instant::now())?)?;
        beyond(settings.remaining(self.elapsed())?)
    }

    fn reading(injected: Option<&PathBuf>) -> Option<ClockReading> {
        injected.and_then(|file| meta_door::Door::read(file).ok()).and_then(|text| text.parse().ok())
    }
}

impl fmt::Display for WatchClock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.injected.as_ref().map_or("", |_| Self::MARK))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::WatchClock;
    use crate::request::project::Config;

    const SETTINGS: &str = "[ci]\ntimeout_millis = 20\ngh_millis = 10\n";
    const PAST_TIMEOUT: Duration = Duration::from_millis(10);

    #[test]
    fn a_pause_stops_one_gh_limit_before_the_timeout_and_remaining_ends_at_the_timeout() {
        let settings = toml::from_str::<Config>(SETTINGS).unwrap().ci_settings();
        let clock = WatchClock::start(&settings);
        let first = (clock.remaining(&settings), clock.pause(Duration::MAX, &settings));
        let second = clock.pause(Duration::MAX, &settings);
        std::thread::sleep(PAST_TIMEOUT);
        assert!(first == (Some(()), Some(())) && second.is_none() && clock.remaining(&settings).is_none());
    }
}
