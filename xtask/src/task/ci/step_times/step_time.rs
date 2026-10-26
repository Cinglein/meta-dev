use std::fmt;
use std::time::Duration;

use crate::task::ci::ci_step::CiStep;

#[derive(Clone, Copy, Debug)]
pub struct StepTime {
    step: CiStep,
    elapsed: Duration,
}

impl StepTime {
    pub fn new(step: CiStep, elapsed: Duration) -> StepTime {
        StepTime { step, elapsed }
    }
}

impl fmt::Display for StepTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {:.1?}", self.step, self.elapsed)
    }
}
