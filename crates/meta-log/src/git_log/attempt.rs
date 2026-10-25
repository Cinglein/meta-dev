use std::fmt;
use std::hash::BuildHasher;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt(u32);

impl Attempt {
    const LIMIT: u32 = 8;
    const BACKOFF: Duration = Duration::from_millis(20);
    const JITTER_STEPS: u64 = 1000;

    pub fn sequence() -> impl Iterator<Item = Self> {
        (0..Self::LIMIT).map(Self)
    }

    pub fn last() -> Self {
        Self(Self::LIMIT)
    }

    pub fn pause(self) {
        let noise = std::collections::hash_map::RandomState::new().hash_one(self.0) % Self::JITTER_STEPS;
        let jitter = Self::BACKOFF.mul_f64(f64::from(u32::try_from(noise).unwrap_or_default()) / f64::from(1000u32));
        std::thread::sleep(Self::BACKOFF * 2u32.saturating_pow(self.0) + jitter);
    }
}

impl fmt::Display for Attempt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
