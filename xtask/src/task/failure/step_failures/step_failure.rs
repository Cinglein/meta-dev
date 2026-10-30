use std::fmt;

use task::{CheckName, FailureSet};

use crate::task::ci::CiStep;
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

    pub fn found(&self) -> FailureSet {
        let job = CheckName::from(self.step);
        self.failure.found(&job).or_infra(job)
    }
}

impl fmt::Display for StepFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ci: {} failed: {}", self.step, self.failure)
    }
}
