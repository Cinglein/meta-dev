mod ci_step;
mod head_branch;
mod local_gate;
mod log_source;
mod run_kind;
mod standing_log;
mod step_times;
mod stranding;
mod tracked_work;
mod untracked;

pub use local_gate::{Busy, LocalGate, OutsideConfig, Posting, Unready};
pub use log_source::LogSource;
pub use untracked::Untracked;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use strum::IntoEnumIterator;

use crate::task::BuildDirectory;
use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::surfaces::Surfaces;
pub use ci_step::CiStep;
use run_kind::RunKind;
use standing_log::StandingLog;
use step_times::StepTimes;
use stranding::Stranding;
use task::Branch;
use task::CommandLine;
use task::Ledger;
use task::Policy;
use task::Selection;
use task::Workspace;
use tracked_work::TrackedWork;

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
                let read = || self.log.snapshot(&StandingLog::git(workspace.root()));
                let prior = || PriorPolicy::text_at_merge_base(workspace.root(), policy.branch(Branch::Base));
                let run = RunKind::from_ambient(&CommandLine::ambient());
                let tracked = BuildDirectory::read(workspace.root())
                    .and_then(|config| config.checked())
                    .map_err(Failure::from)
                    .and_then(|()| TrackedWork::refuse_untracked(&run, read));
                let stepped = tracked
                    .is_ok()
                    .then(CiStep::iter)
                    .into_iter()
                    .flatten()
                    .try_for_each(|step| times.record(step, step.timed(workspace, selection, policy, &mut ledger)));
                let outcome = tracked
                    .and(stepped)
                    .and_then(|()| StandingLog::enforcing(&run, &mut ledger, prior))
                    .and_then(|()| StandingLog::counted(read, policy, &mut ledger, &run.link()))
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
