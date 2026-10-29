use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use task::{FailureSet, Ledger};

use crate::task::ci::LocalGate;
use crate::task::ci::step_times::StepTimes;
use crate::task::failure::Failure;

#[derive(Debug, Default)]
pub struct Attribution {
    times: Option<PathBuf>,
    failures: Option<PathBuf>,
}

impl Attribution {
    pub const FAILURES: &str = "META_CI_FAILURES";

    pub fn new(times: Option<PathBuf>, failures: Option<PathBuf>) -> Attribution {
        Attribution { times, failures }
    }

    pub fn ambient() -> Attribution {
        Attribution::new(
            env::var_os(LocalGate::TIMES).map(PathBuf::from),
            env::var_os(Attribution::FAILURES).map(PathBuf::from),
        )
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
    use task::{FailureKind, Ledger};

    use super::Attribution;
    use crate::task::ci::step_times::StepTimes;
    use crate::task::failure::Failure;

    const GATE: &str = "gate";

    #[test]
    fn a_failure_with_no_structure_is_infra_and_a_pass_writes_an_empty_set() {
        let bare = Failure::from(task::Failure::NoWorkspace);
        let found = bare.found().or_infra(StepTimes::default().failed_job());
        let infra = FailureKind::Infra { job: GATE.into() };
        let silent = Attribution::default().attributed(Some(&bare), &StepTimes::default(), &Ledger::default());
        assert_eq!(found.kinds(), [infra]);
        assert!(silent.is_ok());
    }
}
