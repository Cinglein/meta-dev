mod step_time;

use std::fmt;
use std::time::Duration;

use crate::task::ci::ci_step::CiStep;
use crate::task::failure::Failure;
use step_time::StepTime;

#[derive(Debug, Default)]
pub struct StepTimes {
    passed: Vec<StepTime>,
    failed: Vec<CiStep>,
}

impl StepTimes {
    pub fn reported(failures: Vec<(CiStep, Failure)>) -> Result<(), Failure> {
        let mut failures = failures.into_iter();
        let last = failures.next_back();
        failures.for_each(|(step, failure)| eprintln!("ci: {step} failed: {failure}"));
        last.map_or(Ok(()), |(_, failure)| Err(failure))
    }

    pub fn record(&mut self, step: CiStep, outcome: Result<Duration, Failure>) -> Result<(), Failure> {
        match outcome {
            Ok(elapsed) => {
                self.passed.push(StepTime::new(step, elapsed));
                Ok(())
            }
            Err(failure) => {
                self.failed.push(step);
                Err(failure)
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

    use super::StepTimes;
    use crate::task::ci::ci_step::CiStep;
    use crate::task::ci::{Ci, LogSource};
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;

    const SECOND: Duration = Duration::from_secs(1);
    const SHOWN: &str = "fmt 1.0s, clippy failed, test failed";
    const TIMES: &str = "times";

    impl StepTimes {
        fn failing(&mut self, step: CiStep) -> Result<(), Failure> {
            self.record(step, Err(Failure::from(task::Failure::NoWorkspace)))
        }
    }

    #[test]
    fn passed_steps_show_their_time_a_failed_step_fails_the_gate_and_a_gate_child_records_them() {
        let (fixture, mut times) = (Fixture::new(), StepTimes::default());
        assert!(times.record(CiStep::Fmt, Ok(SECOND)).is_ok());
        assert!(times.failing(CiStep::Clippy).is_err() && times.failing(CiStep::Test).is_err());
        let failed = |step: CiStep| (step, Failure::from(task::Failure::Usage(step.to_string())));
        let reported = StepTimes::reported(vec![failed(CiStep::Clippy), failed(CiStep::Test)]);
        assert!(StepTimes::reported(vec![]).is_ok() && reported.is_err_and(|failure| failure.to_string() == "test"));
        Ci::new(None, None, Some(fixture.path(TIMES)), LogSource::Origin).recorded(&times).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(TIMES)).ok().as_deref(), Some(SHOWN));
    }
}
