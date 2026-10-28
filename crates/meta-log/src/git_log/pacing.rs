use std::time::Duration;

use super::attempt::Attempt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pacing {
    Jittered,
    #[cfg(test)]
    Immediate,
}

impl Pacing {
    pub fn pause(self, attempt: Attempt) -> Duration {
        match self {
            Self::Jittered => attempt.pause(),
            #[cfg(test)]
            Self::Immediate => Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Attempt, Pacing};

    #[test]
    fn jittered_pacing_pauses_as_each_attempt_and_immediate_pacing_never() {
        let paused = |pacing: Pacing| Attempt::sequence().map(move |attempt| pacing.pause(attempt));
        assert!(paused(Pacing::Jittered).any(|pause| !pause.is_zero()));
        assert!(paused(Pacing::Immediate).all(|pause| pause.is_zero()));
    }
}
