mod step_failure;
mod step_time;

use std::fmt;
use std::time::Duration;

use crate::task::ci::ci_step::CiStep;
use crate::task::failure::Failure;
use step_failure::StepFailure;
use step_time::StepTime;

#[derive(Debug, Default)]
pub struct StepTimes {
    passed: Vec<StepTime>,
    failed: Vec<CiStep>,
}

impl StepTimes {
    pub fn reported(mut failures: Vec<StepFailure>) -> Result<(), Failure> {
        let last = failures.pop();
        eprint!("{}", failures.iter().fold(String::new(), |shown, failure| shown + &failure.to_string() + "\n"));
        last.map_or(Ok(()), |failure| Err(failure.into_failure()))
    }

    pub fn record(&mut self, step: CiStep, outcome: Result<Duration, Failure>) -> Option<StepFailure> {
        match outcome {
            Ok(elapsed) => {
                self.passed.push(StepTime::new(step, elapsed));
                None
            }
            Err(failure) => {
                self.failed.push(step);
                Some(StepFailure::new(step, failure))
            }
        }
    }
}

impl fmt::Display for StepTimes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let failed = self.failed.iter().map(|step| format!("{step} failed"));
        formatter
            .write_str(&self.passed.iter().map(ToString::to_string).chain(failed).collect::<Vec<String>>().join(", "))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::{StepFailure, StepTimes};
    use crate::task::ci::ci_step::CiStep;
    use crate::task::ci::{Ci, LogSource};
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;

    const SECOND: Duration = Duration::from_secs(1);
    const SHOWN: &str = "fmt 1.0s, clippy failed, test failed";
    const TIMES: &str = "times";

    impl StepTimes {
        fn failing(&mut self, step: CiStep) -> Option<StepFailure> {
            self.record(step, Err(Failure::from(task::Failure::NoWorkspace)))
        }
    }

    #[test]
    fn passed_steps_show_their_time_a_failed_step_fails_the_gate_and_a_gate_child_records_them() {
        let (fixture, mut times) = (Fixture::new(), StepTimes::default());
        assert!(times.record(CiStep::Fmt, Ok(SECOND)).is_none());
        let failures = [CiStep::Clippy, CiStep::Test].map(|step| times.failing(step)).into_iter().flatten().collect();
        assert!(StepTimes::reported(failures).is_err());
        Ci::new(None, None, Some(fixture.path(TIMES)), LogSource::Origin).recorded(&times).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(TIMES)).ok().as_deref(), Some(SHOWN));
    }
}
