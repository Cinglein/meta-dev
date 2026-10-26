mod seconds;

use seconds::Seconds;
use serde::Deserialize;

use task::Millis;

#[derive(Debug, Deserialize)]
pub struct PhaseResult {
    duration: Seconds,
}

impl PhaseResult {
    #[must_use]
    pub fn total(phases: &[PhaseResult]) -> Millis {
        phases.iter().map(|phase| phase.duration.millis()).fold(Millis::default(), Millis::plus)
    }
}
