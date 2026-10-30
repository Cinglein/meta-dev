mod step_failure;

use std::fmt;

use task::FailureSet;

pub use step_failure::StepFailure;

#[derive(Debug)]
pub struct StepFailures {
    first: StepFailure,
    rest: Vec<StepFailure>,
}

impl StepFailures {
    pub fn new(mut failures: Vec<StepFailure>) -> Option<StepFailures> {
        (!failures.is_empty()).then(|| {
            let first = failures.remove(0);
            StepFailures { first, rest: failures }
        })
    }

    pub fn found(&self) -> FailureSet {
        let all = std::iter::once(&self.first).chain(&self.rest);
        all.map(StepFailure::found).fold(FailureSet::default(), |set, found| set.joined(&found))
    }
}

impl fmt::Display for StepFailures {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let all = std::iter::once(&self.first).chain(&self.rest);
        formatter.write_str(&all.map(ToString::to_string).collect::<Vec<String>>().join("\n"))
    }
}
