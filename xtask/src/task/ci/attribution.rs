use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use task::{FailureKind, FailureSet};

use crate::task::ci::LocalGate;
use crate::task::ci::step_outcomes::StepOutcomes;
use crate::task::ci::step_times::StepTimes;
use crate::task::failure::Failure;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Attribution {
    times: Option<PathBuf>,
    failures: Option<PathBuf>,
}

impl Attribution {
    pub const FAILURES: &str = "META_CI_FAILURES";

    pub fn new(times: Option<PathBuf>, failures: Option<PathBuf>) -> Attribution {
        Attribution { times, failures }
    }

    pub fn from_ambient(ambient: &BTreeMap<String, OsString>) -> Attribution {
        let path = |name: &str| ambient.get(name).map(PathBuf::from);
        Attribution::new(path(LocalGate::TIMES), path(Self::FAILURES))
    }

    pub fn begin(&self) -> Result<(), Failure> {
        self.failures.as_ref().map_or(Ok(()), |path| FailureSet::begin(path)).map_err(Failure::Failures)
    }

    pub fn recorded(&self, times: &StepTimes) -> Result<(), Failure> {
        self.times
            .as_ref()
            .map_or(Ok(()), |path| File::create(path).and_then(|mut file| write!(file, "{times}")))
            .map_err(Failure::Summary)
    }

    pub fn attributed(&self, failure: &Failure, times: &StepTimes) -> Result<(), Failure> {
        let job = times.failed_job();
        self.failures
            .as_ref()
            .map_or(Ok(()), |path| failure.found(&job).or_infra(job).write(path))
            .map_err(Failure::Failures)
    }

    pub fn closed(&self, steps: &StepOutcomes) -> Result<(), Failure> {
        let path = self.failures.as_ref().ok_or(task::Failure::Unset(Self::FAILURES))?;
        let first = if steps.first_passed() { FailureSet::default() } else { FailureSet::read(path)? };
        let later = steps.later_failures().map(|job| FailureKind::Infra { job });
        later.fold(first, FailureSet::recording).write(path).map_err(Failure::Failures)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::{env, fs, process};

    use task::{FailureKind, FailureSet};

    use super::Attribution;
    use crate::task::ci::LocalGate;
    use crate::task::ci::step_outcomes::StepOutcomes;
    use crate::task::ci::step_times::StepTimes;
    use crate::task::failure::Failure;

    const SCRATCH: &str = "attribution-";
    const CI: &str = "ci";
    const CONSUMER: &str = "consumer";
    const PASSED: &str = "ci=success,consumer=success";
    const LATER: &str = "ci=success,consumer=failure,stacked=skipped";
    const FIRST: &str = "ci=failure,consumer=skipped";

    #[test]
    fn the_ambient_names_pick_the_two_output_paths() {
        let named = |name: &str| (name.to_owned(), OsString::from(name));
        let ambient = BTreeMap::from([named(LocalGate::TIMES), named(Attribution::FAILURES)]);
        let path = |name: &str| Some(PathBuf::from(name));
        let expected = Attribution::new(path(LocalGate::TIMES), path(Attribution::FAILURES));
        assert_eq!(Attribution::from_ambient(&ambient), expected);
    }

    #[test]
    fn a_begun_file_stays_unattributed_through_a_pass_and_only_the_close_step_writes_the_whole_job() {
        let file = env::temp_dir().join(format!("{SCRATCH}{}", process::id()));
        let attribution = Attribution::new(None, Some(file.clone()));
        let closed = |steps: &str| attribution.closed(&steps.parse::<StepOutcomes>().unwrap());
        let read = || FailureSet::read(&file).ok().map(|set| set.kinds().to_vec());
        let infra = |job: &str| Some(vec![FailureKind::Infra { job: job.into() }]);
        attribution.begin().unwrap();
        let begun = [closed(FIRST).is_err(), read().is_none()];
        attribution.attributed(&Failure::NoLog, &StepTimes::default()).unwrap();
        let steps = [FIRST, LATER, PASSED].map(|steps| closed(steps).map(|()| read()).unwrap());
        fs::remove_file(&file).unwrap();
        assert_eq!(begun, [true, true]);
        assert_eq!(steps, [infra(CI), infra(CONSUMER), Some(vec![])]);
    }
}
