use std::fmt;

use crate::task::ci::ci_step::CiStep;
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct StepFailure {
    step: CiStep,
    failure: Failure,
}

impl StepFailure {
    pub fn new(step: CiStep, failure: Failure) -> StepFailure {
        StepFailure { step, failure }
    }

    pub fn into_failure(self) -> Failure {
        self.failure
    }
}

impl fmt::Display for StepFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ci: {} failed: {}", self.step, self.failure)
    }
}
