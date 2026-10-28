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

    pub fn rest(&self, pause: Duration) {
        let left = self.overall.map_or(pause, |overall| overall.saturating_duration_since(Instant::now()));
        std::thread::sleep(pause.min(left));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::Budget;

    const SHORT: Duration = Duration::from_millis(5);
    const LONG: Duration = Duration::from_secs(5);

    #[test]
    fn each_call_starts_its_own_limit_inside_the_overall_deadline_and_rests_end_there() {
        let budget = Budget::new(SHORT);
        let started = Instant::now();
        Budget::new(LONG).within(started + SHORT).rest(LONG);
        assert!(budget.deadline() > Instant::now() && (SHORT..LONG).contains(&started.elapsed()));
        assert_eq!(Budget::new(LONG).within(started).deadline(), started);
    }
}
