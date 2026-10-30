mod attribution;
mod ci_step;
mod head_branch;
mod local_gate;
mod log_source;
mod run_kind;
mod standing_log;
mod step_outcomes;
mod step_times;
mod stranding;
mod tracked_build_output;
mod tracked_work;
mod untracked;

pub use local_gate::{Busy, LocalGate, OutsideConfig, Posting, Unready};
pub use log_source::LogSource;
pub use untracked::Untracked;

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use strum::IntoEnumIterator;

use crate::task::BuildDirectory;
use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::surfaces::Surfaces;
use attribution::Attribution;
pub use ci_step::CiStep;
use run_kind::RunKind;
use standing_log::StandingLog;
use step_outcomes::StepOutcomes;
use step_times::StepTimes;
use stranding::Stranding;
use task::Branch;
use task::CommandLine;
use task::Ledger;
use task::Policy;
use task::Selection;
use task::Workspace;
use tracked_build_output::TrackedBuildOutput;
use tracked_work::TrackedWork;

pub struct Ci {
    summary: Option<PathBuf>,
    gate: Option<LocalGate>,
    attribution: Attribution,
    log: LogSource,
}

impl Ci {
    pub fn new(summary: Option<PathBuf>, gate: Option<LocalGate>, log: LogSource) -> Ci {
        Ci { summary, gate, attribution: Attribution::from_ambient(&CommandLine::ambient()), log }
    }

    pub const CLOSE: &str = "--close";

    pub fn close() -> Result<(), Failure> {
        let steps = env::var(StepOutcomes::AMBIENT).map_err(|_| task::Failure::Unset(StepOutcomes::AMBIENT))?;
        Attribution::from_ambient(&CommandLine::ambient()).closed(&steps.parse()?)
    }

    pub fn run(&self, workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
        self.gate.as_ref().map_or_else(
            || {
                self.attribution.begin()?;
                println!("ci: checking {selection}");
                let started = Instant::now();
                let mut times = StepTimes::default();
                let mut ledger = Ledger::default();
                let read = || {
                    let log = StandingLog::git(workspace.root()).map_err(Failure::StandingLog)?;
                    self.log.snapshot(&log)
                };
                let prior = || PriorPolicy::text_at_merge_base(workspace.root(), policy.branch(Branch::Base));
                let run = RunKind::from_ambient(&CommandLine::ambient());
                let tracked = BuildDirectory::read(workspace.root())
                    .and_then(|config| config.checked())
                    .map_err(Failure::from)
                    .and_then(|()| TrackedBuildOutput::refuse(workspace.root()))
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
                let recorded = self.attribution.recorded(&times);
                let attributed =
                    outcome.as_ref().err().map_or(Ok(()), |failure| self.attribution.attributed(failure, &times));
                Surfaces::announce(workspace.root(), policy, self.summary.as_deref())
                    .unwrap_or_else(|failure| eprintln!("surfaces: {failure}"));
                outcome.and(recorded).and(attributed)
            },
            |gate| gate.run(policy, self.summary.as_deref()),
        )
    }
}
