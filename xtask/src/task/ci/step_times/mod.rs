mod step_time;

use std::fmt;
use std::time::Duration;

use crate::task::ci::ci_step::CiStep;
use crate::task::failure::Failure;
use step_time::StepTime;

#[derive(Debug, Default)]
pub struct StepTimes {
    passed: Vec<StepTime>,
    failed: Option<CiStep>,
}

impl StepTimes {
    pub fn record(&mut self, step: CiStep, outcome: Result<Duration, Failure>) -> Result<(), Failure> {
        match outcome {
            Ok(elapsed) => {
                self.passed.push(StepTime::new(step, elapsed));
                Ok(())
            }
            Err(failure) => {
                self.failed = Some(step);
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
    use meta_door::Door;
    use std::time::Duration;

    use super::StepTimes;
    use crate::task::ci::ci_step::CiStep;
    use crate::task::ci::{Ci, LogSource};
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;

    const SECOND: Duration = Duration::from_secs(1);
    const SHOWN: &str = "fmt 1.0s, clippy failed";
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
        assert!(times.failing(CiStep::Clippy).is_err());
        Ci::new(None, None, Some(fixture.path(TIMES)), LogSource::Origin).recorded(&times).unwrap();
        assert_eq!(Door::read(fixture.path(TIMES)).ok().as_deref(), Some(SHOWN));
    }
}
