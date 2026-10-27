use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub struct ClockReading(Duration);

impl ClockReading {
    pub fn elapsed(self) -> Duration {
        self.0
    }
}

impl FromStr for ClockReading {
    type Err = ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse().map(|millis| Self(Duration::from_millis(millis)))
    }
}
