use std::hash::BuildHasher;
use std::time::{Duration, Instant};

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
    pub fn within<Found, Failure>(
        deadline: Instant,
        budget: &Budget,
        mut found: impl FnMut() -> Result<Option<Found>, Failure>,
    ) -> Option<Result<Found, Failure>> {
        let bounded = budget.within(deadline);
        let attempts = Self::sequence().take_while(|_| deadline.checked_duration_since(Instant::now()).is_some());
        attempts.into_iter().find_map(|attempt| attempt.settled(found(), &bounded))
    }

    fn settled<Found, Failure>(
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
    use crate::log_failure::LogFailure;
    use meta_run::Budget;
    use std::time::{Duration, Instant};
    const PATIENCE: Duration = Duration::from_millis(64);
    const SLACK: Duration = Duration::from_secs(2);

    #[test]
    fn a_retry_that_never_settles_gives_up_expired_by_a_deadline_each_call_takes_afresh() {
        let (budget, started) = (Budget::new(PATIENCE), Instant::now());
        let deadline = budget.deadline();
        let gave_up = Attempt::within(deadline, &budget, || Ok::<Option<()>, ()>(None));
        let expired = matches!(LogFailure::gave_up(deadline), LogFailure::Expired);
        assert!(gave_up.is_none() && expired && started.elapsed() < PATIENCE + SLACK);
        assert!(budget.deadline() > deadline);
    }

    #[test]
    fn pauses_are_jittered_within_the_window() {
        let pauses: Vec<_> = Attempt::sequence().map(Attempt::pause).collect();
        assert!(pauses.iter().all(|pause| *pause < Attempt::MAX_WINDOW));
        assert!(pauses.iter().any(|pause| !pause.is_zero()));
    }
}
