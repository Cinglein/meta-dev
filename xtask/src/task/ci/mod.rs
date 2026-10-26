mod ci_step;

use std::time::Instant;

use strum::IntoEnumIterator;

use crate::task::failure::Failure;
use ci_step::CiStep;
use meta_lints::Project;
use task::Policy;
use task::Selection;
use task::Workspace;

pub struct Ci;

impl Ci {
    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        project: &Project,
    ) -> Result<(), Failure> {
        println!("ci: checking {selection}");
        let started = Instant::now();
        let outcome = CiStep::iter().try_for_each(|step| step.timed(workspace, selection, policy, project));
        println!("ci: total {:.1?}", started.elapsed());
        project
            .changed_surfaces()
            .and_then(|changed| changed.announce())
            .unwrap_or_else(|failure| eprintln!("surfaces: {failure}"));
        outcome
    }
}
