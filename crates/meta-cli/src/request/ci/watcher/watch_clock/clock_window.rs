use std::fmt;
use std::ops::ControlFlow;
use std::time::Duration;

use crate::request::ci::watcher::note::Note;
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

    pub fn poll(
        self,
        refused: Option<impl fmt::Display>,
        wait: Duration,
        settings: &CiSettings,
    ) -> ControlFlow<Option<Note>, Duration> {
        let unchecked = refused.map(|reason| ControlFlow::Break(Some(Note::unchecked(self.elapsed, reason))));
        let pause = self.left(settings, settings.gh_limit()).map(|window| ControlFlow::Continue(wait.min(window)));
        unchecked.or(pause).unwrap_or(ControlFlow::Break(None))
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

    use super::{ClockWindow, ControlFlow::Break, ControlFlow::Continue, Note};
    use crate::request::ci::watcher::watch_clock::WatchClock;
    use crate::request::project::{CiSettings, Config};

    const SETTINGS: &str = "[ci]\ntimeout_millis = 20\ngh_millis = 10\n";
    const SPENT: &str = "[ci]\ntimeout_millis = 20\ngh_millis = 20\n";
    const STEP: Duration = Duration::from_millis(1);
    const READABLE: Option<&str> = None;
    const REFUSED: Option<&str> = Some("remote URL names no GitHub repository");
    const PAUSES: [(u64, Option<u64>, Option<u64>); 4] =
        [(0, Some(20), Some(10)), (5, Some(15), Some(5)), (10, Some(10), None), (0, None, None)];
    const REMAINING: [(u64, Option<u64>, Option<()>); 3] =
        [(15, Some(5), Some(())), (20, Some(5), None), (5, Some(0), None)];

    #[test]
    fn a_refused_remote_is_unchecked_at_any_clock_and_the_last_poll_ends_one_gh_limit_before_the_timeout() {
        let [settings, spent] =
            [SETTINGS, SPENT].map(|text| CiSettings::try_from(&toml::from_str::<Config>(text).unwrap()).unwrap());
        let ms = Duration::from_millis;
        let at = |at, left: Option<u64>| ClockWindow::new(ms(at), left.map(ms));
        let polled = |refused| PAUSES.map(|(at_ms, left, _)| at(at_ms, left).poll(refused, Duration::MAX, &settings));
        let remaining = REMAINING.map(|(elapsed, left, _)| at(elapsed, left).remaining(&settings));
        let unchecked = PAUSES.map(|(at, ..)| Break(REFUSED.map(|reason| Note::unchecked(ms(at), reason))));
        assert_eq!(polled(READABLE), PAUSES.map(|(.., pause)| pause.map(ms).map_or(Break(None), Continue)));
        assert!(polled(REFUSED) == unchecked && remaining == REMAINING.map(|(.., remaining)| remaining));
        let started = WatchClock::start(&settings);
        let spent = WatchClock::start(&spent).poll(READABLE, STEP, &spent);
        assert!(started.poll(READABLE, STEP, &settings).is_continue() && started.elapsed() >= STEP && spent.is_break());
    }
}
