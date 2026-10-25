use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub struct Elapsed(Duration);

impl Elapsed {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s", self.0.as_secs())
    }
}
