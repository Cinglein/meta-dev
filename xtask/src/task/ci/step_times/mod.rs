mod step_time;

use std::fmt;
use std::time::Duration;

use crate::task::ci::ci_step::CiStep;
use crate::task::failure::{Failure, StepFailure, StepFailures};
use step_time::StepTime;

#[derive(Debug, Default)]
pub struct StepTimes {
    passed: Vec<StepTime>,
    failed: Vec<CiStep>,
}

impl StepTimes {
    pub fn stepped(
        &mut self,
        steps: impl IntoIterator<Item = CiStep>,
        mut run: impl FnMut(CiStep) -> Result<Duration, Failure>,
    ) -> Result<(), Failure> {
        let failures = steps.into_iter().filter_map(|step| self.record(step, run(step))).collect();
        Self::reported(failures)
    }

    fn reported(failures: Vec<StepFailure>) -> Result<(), Failure> {
        StepFailures::new(failures).map_or(Ok(()), |failures| Err(Failure::Steps(Box::new(failures))))
    }

    fn record(&mut self, step: CiStep, outcome: Result<Duration, Failure>) -> Option<StepFailure> {
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
    use std::cell::Cell;
    use std::fs;
    use std::time::Duration;

    use super::StepTimes;
    use crate::task::ci::ci_step::CiStep;
    use crate::task::ci::{Ci, LogSource};
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;

    const SECOND: Duration = Duration::from_secs(1);
    const STEPS: [CiStep; 4] = [CiStep::Fmt, CiStep::Clippy, CiStep::Test, CiStep::Lint];
    const SHOWN: &str = "fmt 1.0s, lint 1.0s, clippy failed, test failed";
    const LISTED: &str = "ci: clippy failed: clippy\nci: test failed: test";
    const TIMES: &str = "times";

    impl StepTimes {
        fn outcome(step: CiStep) -> Result<Duration, Failure> {
            match step {
                CiStep::Clippy | CiStep::Test => Err(Failure::from(task::Failure::Usage(step.to_string()))),
                _ => Ok(SECOND),
            }
        }
    }

    #[test]
    fn the_loop_runs_past_failing_steps_and_fails_listing_every_one_with_its_step() {
        let (ran, mut times) = (Cell::new(0), StepTimes::default());
        let stepped = times.stepped(STEPS, |step| {
            ran.set(ran.get() + 1);
            StepTimes::outcome(step)
        });
        assert_eq!((ran.get(), stepped.err().map(|failure| failure.to_string())), (4, Some(LISTED.to_owned())));
    }

    #[test]
    fn no_failing_step_passes_and_a_gate_child_records_every_step_time() {
        let (fixture, mut times) = (Fixture::new(), StepTimes::default());
        assert!(times.stepped([], StepTimes::outcome).is_ok() && times.stepped(STEPS, StepTimes::outcome).is_err());
        Ci::new(None, None, Some(fixture.path(TIMES)), LogSource::Origin).recorded(&times).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(TIMES)).ok().as_deref(), Some(SHOWN));
    }
}
