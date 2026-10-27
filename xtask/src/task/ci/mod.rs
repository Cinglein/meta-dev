mod ci_step;
mod local_gate;
mod log_source;
mod standing_log;
mod step_times;
mod stranding;

pub use local_gate::{Busy, LocalGate, OutsideConfig, Posting, Unready};
pub use log_source::LogSource;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use strum::IntoEnumIterator;

use crate::task::failure::Failure;
use crate::task::surfaces::Surfaces;
pub use ci_step::CiStep;
use standing_log::StandingLog;
use step_times::StepTimes;
use stranding::Stranding;
use task::Ledger;
use task::Policy;
use task::Selection;
use task::Workspace;

pub struct Ci {
    summary: Option<PathBuf>,
    gate: Option<LocalGate>,
    record: Option<PathBuf>,
    log: LogSource,
}

impl Ci {
    pub fn new(summary: Option<PathBuf>, gate: Option<LocalGate>, record: Option<PathBuf>, log: LogSource) -> Ci {
        Ci { summary, gate, record, log }
    }

    pub fn run(&self, workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
        self.gate.as_ref().map_or_else(
            || {
                println!("ci: checking {selection}");
                let started = Instant::now();
                let mut times = StepTimes::default();
                let mut ledger = Ledger::default();
                let log = StandingLog::git(workspace.root());
                let read = || self.log.snapshot(&log);
                let outcome = CiStep::iter()
                    .try_for_each(|step| times.record(step, step.timed(workspace, selection, policy, &mut ledger)))
                    .and_then(|()| StandingLog::counted(read, policy, &mut ledger))
                    .and_then(|()| Ok(ledger.checked(policy, selection)?))
                    .and_then(|()| Stranding::refuse_stranded(workspace.root(), policy, read));
                println!("ci: total {:.1?}", started.elapsed());
                self.recorded(&times)?;
                Surfaces::announce(workspace.root(), policy, self.summary.as_deref())
                    .unwrap_or_else(|failure| eprintln!("surfaces: {failure}"));
                outcome
            },
            |gate| gate.run(policy, self.summary.as_deref()),
        )
    }

    fn recorded(&self, times: &StepTimes) -> Result<(), Failure> {
        self.record
            .as_ref()
            .map_or(Ok(()), |path| File::create(path).and_then(|mut file| write!(file, "{times}")))
            .map_err(Failure::Summary)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::ci_step::CiStep;
    use super::step_times::StepTimes;
    use super::{Ci, LogSource};
    use crate::task::fixture::Fixture;

    const TIMES: &str = "times";
    const SHOWN: &str = "lint 1.0s";

    #[test]
    fn a_gate_child_records_its_step_times() {
        let fixture = Fixture::new();
        let mut times = StepTimes::default();
        times.record(CiStep::Lint, Ok(Duration::from_secs(1))).unwrap();
        Ci::new(None, None, Some(fixture.path(TIMES)), LogSource::Origin).recorded(&times).unwrap();
        assert_eq!(fs::read_to_string(fixture.path(TIMES)).ok().as_deref(), Some(SHOWN));
    }
}
