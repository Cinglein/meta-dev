use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use task::{FailureSet, Ledger};

use crate::task::ci::LocalGate;
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

    pub fn recorded(&self, times: &StepTimes) -> Result<(), Failure> {
        self.times
            .as_ref()
            .map_or(Ok(()), |path| File::create(path).and_then(|mut file| write!(file, "{times}")))
            .map_err(Failure::Summary)
    }

    pub fn attributed(&self, failure: Option<&Failure>, times: &StepTimes, ledger: &Ledger) -> Result<(), Failure> {
        self.failures
            .as_ref()
            .map_or(Ok(()), |path| {
                failure
                    .map_or_else(FailureSet::default, |failure| {
                        failure.found().joined(ledger.failures()).or_infra(times.failed_job())
                    })
                    .write(path)
            })
            .map_err(Failure::Failures)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::{env, fs, process};

    use task::{FailureKind, FailureSet, Ledger};

    use super::Attribution;
    use crate::task::ci::LocalGate;
    use crate::task::ci::step_times::StepTimes;
    use crate::task::failure::Failure;

    const GATE: &str = "gate";
    const SCRATCH: &str = "attribution-";

    #[test]
    fn the_ambient_names_pick_the_two_output_paths() {
        let named = |name: &str| (name.to_owned(), OsString::from(name));
        let ambient = BTreeMap::from([named(LocalGate::TIMES), named(Attribution::FAILURES)]);
        let path = |name: &str| Some(PathBuf::from(name));
        let expected = Attribution::new(path(LocalGate::TIMES), path(Attribution::FAILURES));
        assert_eq!(Attribution::from_ambient(&ambient), expected);
    }

    #[test]
    fn a_pass_writes_an_empty_set_and_a_bare_failure_writes_infra_unless_the_ledger_holds_failures() {
        let file = env::temp_dir().join(format!("{SCRATCH}{}", process::id()));
        let attribution = Attribution::new(None, Some(file.clone()));
        let bare = Failure::from(task::Failure::NoWorkspace);
        let written = |failure: Option<&Failure>, ledger: &Ledger| {
            attribution.attributed(failure, &StepTimes::default(), ledger).unwrap();
            FailureSet::read(&file).unwrap()
        };
        let mut held = Ledger::default();
        held.fail(FailureSet::default().recording(FailureKind::Infra { job: SCRATCH.into() }));
        let sets = [written(None, &held), written(Some(&bare), &Ledger::default()), written(Some(&bare), &held)];
        fs::remove_file(&file).unwrap();
        let kinds = sets.map(|set| set.kinds().to_vec());
        assert_eq!(kinds[..2], [vec![], vec![FailureKind::Infra { job: GATE.into() }]]);
        assert_eq!(kinds[2], [FailureKind::Infra { job: SCRATCH.into() }]);
    }
}
