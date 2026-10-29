use std::time::Duration;

use serde::Deserialize;

use super::millis::Millis;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct PollSchedule {
    #[serde(rename = "expected_millis")]
    expected: Millis,
    #[serde(rename = "backoff_millis")]
    backoff: Millis,
    #[serde(rename = "backoff_cap_millis")]
    cap: Millis,
}

impl PollSchedule {
    const SCHEDULE_LEN: usize = 10;

    pub fn waits(self, learned: Option<Duration>) -> impl Iterator<Item = Duration> {
        let first = learned.unwrap_or(self.expected.duration());
        let doubled = std::iter::successors(Some(self.backoff.duration()), |wait| Some(wait.saturating_add(*wait)));
        let capped = doubled.map(move |wait| wait.min(self.cap.duration()));
        std::iter::once(first).chain(capped).take(Self::SCHEDULE_LEN - 1).chain([Duration::MAX])
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::request::project::{CiSettings, Config};

    const SETTINGS: &str = "[ci]\nexpected_millis = 7\nbackoff_millis = 3\nbackoff_cap_millis = 20\n";
    const LEARNED: Duration = Duration::from_millis(5);
    const WAITS: [u64; 9] = [7, 3, 6, 12, 20, 20, 20, 20, 20];

    const DEFAULT_LEARNED: Duration = Duration::from_secs(422);
    const LONGEST_GAP: Duration = Duration::from_secs(60);
    const POLLS_CHECKED: usize = 1000;

    #[test]
    fn with_the_defaults_a_run_ending_after_the_expected_time_is_noticed_within_a_minute_until_the_timeout() {
        let settings = CiSettings::try_from(&toml::from_str::<Config>("").unwrap()).unwrap();
        let timeout = settings.remaining(Duration::ZERO).unwrap();
        let waits: Vec<Duration> = settings.poll_schedule().waits(Some(DEFAULT_LEARNED)).take(POLLS_CHECKED).collect();
        let covered = waits.iter().fold(Duration::ZERO, |sum, wait| sum.saturating_add(*wait));
        let slowest = waits.iter().skip(1).max().copied();
        assert!(slowest <= Some(LONGEST_GAP) && covered >= timeout, "{slowest:?} {covered:?}");
    }

    #[test]
    fn the_first_poll_waits_the_expected_run_then_waits_double_to_the_cap_and_the_last_runs_to_the_deadline() {
        let schedule = CiSettings::try_from(&toml::from_str::<Config>(SETTINGS).unwrap()).unwrap().poll_schedule();
        let expected = WAITS.map(Duration::from_millis).into_iter().chain([Duration::MAX]);
        assert_eq!(schedule.waits(None).collect::<Vec<_>>(), expected.collect::<Vec<_>>());
        assert_eq!(schedule.waits(Some(LEARNED)).next(), Some(LEARNED));
    }
}
