use std::str::FromStr;
use std::time::Duration;

use super::super::unreadable::Unreadable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTime(u64);

impl ClockTime {
    const SIXTY: u64 = 60;
    const DAY: u64 = 86_400;

    pub fn duration(self) -> Duration {
        Duration::from_secs(self.0)
    }
}

impl FromStr for ClockTime {
    type Err = Unreadable;

    fn from_str(clock: &str) -> Result<Self, Self::Err> {
        let whole = clock.split_once('.').map_or(clock, |(whole, _)| whole);
        let (days, time) = whole.split_once('-').unwrap_or(("0", whole));
        let days: u64 = days.parse().map_err(|_| Unreadable)?;
        let seconds = time.split(':').try_fold(0, |total: u64, part| {
            part.parse::<u64>().ok().map(|part| total.saturating_mul(Self::SIXTY).saturating_add(part))
        });
        seconds.map(|seconds| Self(days.saturating_mul(Self::DAY).saturating_add(seconds))).ok_or(Unreadable)
    }
}
