mod ci_step;
mod local_gate;
mod step_times;

pub use local_gate::{LocalGate, Posting, Unready};

use std::path::PathBuf;
use std::time::Instant;

use strum::IntoEnumIterator;

use crate::task::failure::Failure;
use crate::task::surfaces::Surfaces;
use ci_step::CiStep;
use step_times::StepTimes;
use task::Policy;
use task::Selection;
use task::Workspace;

pub struct Ci {
    summary: Option<PathBuf>,
    gate: Option<LocalGate>,
}

impl Ci {
    pub fn new(summary: Option<PathBuf>, gate: Option<LocalGate>) -> Ci {
        Ci { summary, gate }
    }

    pub fn run(&self, workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
        let checked = self.gate.as_ref().map(LocalGate::ready).transpose()?;
        println!("ci: checking {selection}");
        let started = Instant::now();
        let mut times = StepTimes::default();
        let outcome = CiStep::iter().try_for_each(|step| times.record(step, step.timed(workspace, selection, policy)));
        println!("ci: total {:.1?}", started.elapsed());
        Surfaces::announce(workspace.root(), policy, self.summary.as_deref())
            .unwrap_or_else(|failure| eprintln!("surfaces: {failure}"));
        let posted = self.gate.as_ref().zip(checked).map_or(Ok(()), |(gate, checked)| gate.post(&checked, &times));
        posted.and(outcome)
    }
}
