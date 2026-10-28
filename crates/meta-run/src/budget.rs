use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Budget {
    per_call: Duration,
    overall: Option<Instant>,
}

impl Budget {
    #[must_use]
    pub fn new(per_call: Duration) -> Self {
        Self { per_call, overall: None }
    }

    #[must_use]
    pub fn within(self, overall: Instant) -> Self {
        Self { per_call: self.per_call, overall: Some(overall) }
    }

    #[must_use]
    pub fn deadline(&self) -> Instant {
        let call = Instant::now() + self.per_call;
        self.overall.map_or(call, |overall| call.min(overall))
    }

    pub fn retried<Value, Failure>(
        &self,
        mut attempt: impl FnMut() -> Result<Value, Failure>,
    ) -> Result<Value, Failure> {
        let left = || self.overall.is_none_or(|overall| Instant::now() < overall);
        attempt().or_else(|failure| left().then(&mut attempt).unwrap_or(Err(failure)))
    }

    pub fn rest(&self, pause: Duration) {
        let left = self.overall.map_or(pause, |overall| overall.saturating_duration_since(Instant::now()));
        std::thread::sleep(pause.min(left));
    }
}
