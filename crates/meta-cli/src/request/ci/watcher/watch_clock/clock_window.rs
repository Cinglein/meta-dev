use std::time::Duration;

use crate::request::project::CiSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockWindow {
    elapsed: Duration,
    to_deadline: Option<Duration>,
}

impl ClockWindow {
    pub fn new(elapsed: Duration, to_deadline: Option<Duration>) -> Self {
        Self { elapsed, to_deadline }
    }

    pub fn remaining(self, settings: &CiSettings) -> Option<()> {
        self.left(settings, Duration::ZERO).map(drop)
    }

    pub fn pause(self, wait: Duration, settings: &CiSettings) -> Option<Duration> {
        self.left(settings, settings.gh_limit()).map(|window| wait.min(window))
    }

    fn left(self, settings: &CiSettings, reserve: Duration) -> Option<Duration> {
        let beyond = |left: Duration| left.checked_sub(reserve).filter(|rest| !rest.is_zero());
        beyond(self.to_deadline?)?;
        beyond(settings.remaining(self.elapsed)?)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ClockWindow;
    use crate::request::ci::watcher::watch_clock::WatchClock;
    use crate::request::project::{CiSettings, Config};

    const SETTINGS: &str = "[ci]\ntimeout_millis = 20\ngh_millis = 10\n";
    const SPENT: &str = "[ci]\ntimeout_millis = 20\ngh_millis = 20\n";
    const STEP: Duration = Duration::from_millis(1);
    const PAUSES: [(u64, Option<u64>, Option<u64>); 4] =
        [(0, Some(20), Some(10)), (5, Some(15), Some(5)), (10, Some(10), None), (0, None, None)];
    const REMAINING: [(u64, Option<u64>, Option<()>); 3] =
        [(15, Some(5), Some(())), (20, Some(5), None), (5, Some(0), None)];

    #[test]
    fn a_pause_stops_one_gh_limit_before_the_timeout_and_remaining_ends_at_the_timeout() {
        let [settings, spent] =
            [SETTINGS, SPENT].map(|text| CiSettings::try_from(&toml::from_str::<Config>(text).unwrap()).unwrap());
        let at = |at, left: Option<u64>| ClockWindow::new(Duration::from_millis(at), left.map(Duration::from_millis));
        let paused = PAUSES.map(|(elapsed, left, _)| at(elapsed, left).pause(Duration::MAX, &settings));
        let remaining = REMAINING.map(|(elapsed, left, _)| at(elapsed, left).remaining(&settings));
        assert_eq!(paused, PAUSES.map(|(_, _, pause)| pause.map(Duration::from_millis)));
        assert_eq!(remaining, REMAINING.map(|(_, _, remaining)| remaining));
        let started = WatchClock::start(&settings);
        let spent = WatchClock::start(&spent).pause(STEP, &spent);
        assert!(started.pause(STEP, &settings).is_some() && started.elapsed() >= STEP && spent.is_none());
    }
}
