use std::fmt;
use std::str::FromStr;

use super::core_share::CoreShare;
use super::unreadable::Unreadable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadAverage(CoreShare);

impl LoadAverage {
    const MARK: &'static str = "load average";
    const FIFTEEN_MINUTE_INDEX: usize = 2;
}

impl FromStr for LoadAverage {
    type Err = Unreadable;

    fn from_str(uptime: &str) -> Result<Self, Self::Err> {
        let (_, averages) =
            uptime.rsplit_once(Self::MARK).and_then(|(_, tail)| tail.split_once(':')).ok_or(Unreadable)?;
        let figure = averages
            .split([' ', ','])
            .filter(|figure| !figure.trim().is_empty())
            .nth(Self::FIFTEEN_MINUTE_INDEX)
            .ok_or(Unreadable)?;
        figure.parse().map(Self).map_err(|_| Unreadable)
    }
}

impl fmt::Display for LoadAverage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} over 15 min", self.0)
    }
}
