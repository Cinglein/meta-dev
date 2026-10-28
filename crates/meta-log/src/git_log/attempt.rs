use std::hash::BuildHasher;
use std::time::Duration;

use meta_run::Budget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt(u32);

impl Attempt {
    const LIMIT: u32 = 16;
    const BASE_WINDOW: Duration = Duration::from_millis(50);
    const MAX_WINDOW: Duration = Duration::from_secs(2);
    const RESOLUTION: u32 = 1000;
    const GROWTH: u32 = 2;

    pub fn sequence() -> impl Iterator<Item = Self> {
        (0..Self::LIMIT).map(Self)
    }

    pub fn pause(self) -> Duration {
        let window = Self::BASE_WINDOW.saturating_mul(Self::GROWTH.saturating_pow(self.0)).min(Self::MAX_WINDOW);
        let draw = std::collections::hash_map::RandomState::new().hash_one((self.0, std::thread::current().id()));
        let fraction = u32::try_from(draw % u64::from(Self::RESOLUTION)).unwrap_or_default();
        window.mul_f64(f64::from(fraction) / f64::from(Self::RESOLUTION))
    }
}

impl Attempt {
    pub fn settled<Found, Failure>(
        self,
        found: Result<Option<Found>, Failure>,
        budget: &Budget,
    ) -> Option<Result<Found, Failure>> {
        let settled = found.transpose();
        settled.is_none().then(|| budget.rest(self.pause()));
        settled
    }
}

#[cfg(test)]
mod tests {
    use super::Attempt;

    #[test]
    fn pauses_are_jittered_within_the_window() {
        let pauses: Vec<_> = Attempt::sequence().map(Attempt::pause).collect();
        assert!(pauses.iter().all(|pause| *pause < Attempt::MAX_WINDOW));
        assert!(pauses.iter().any(|pause| !pause.is_zero()));
    }
}
