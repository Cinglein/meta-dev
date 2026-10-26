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
use task::Selection;
use task::Workspace;
use task::{Branch, Policy};

pub struct Ci {
    summary: Option<PathBuf>,
    gate: Option<LocalGate>,
}

impl Ci {
    pub fn new(summary: Option<PathBuf>, gate: Option<LocalGate>) -> Ci {
        Ci { summary, gate }
    }

    pub fn run(&self, workspace: &Workspace, selection: &Selection, policy: &Policy) -> Result<(), Failure> {
        let checkout = self.gate.as_ref().map(|gate| gate.checkout(workspace.target().path())).transpose()?;
        let checked_out = checkout.as_ref().map(|checkout| checkout.workspace(workspace)).transpose()?;
        let workspace = checked_out.as_ref().unwrap_or(workspace);
        println!("ci: checking {selection}");
        let started = Instant::now();
        let mut times = StepTimes::default();
        let outcome = CiStep::iter().try_for_each(|step| times.record(step, step.timed(workspace, selection, policy)));
        println!("ci: total {:.1?}", started.elapsed());
        let changed = Surfaces::changed(workspace.root(), policy.surfaces(), policy.branch(Branch::Base))
            .and_then(|changed| changed.announce(self.summary.as_deref()).map(|()| changed));
        let posted = match (&self.gate, &checkout) {
            (Some(gate), Some(checkout)) => changed.and_then(|changed| gate.post(checkout, &times, &changed)),
            _ => changed.map(drop).or_else(|failure| {
                eprintln!("surfaces: {failure}");
                Ok(())
            }),
        };
        posted.and(outcome)
    }
}
