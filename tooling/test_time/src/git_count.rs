use git_trace::Spawns;
use task::{Failure, Ledger, Measure, Policy, Selection, Workspace};

use super::TestTime;
use super::execution::Execution;

#[derive(Debug)]
pub struct GitCount;

impl GitCount {
    pub const SWITCH: &'static str = "META_GIT_COUNT";

    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<Spawns, Failure> {
        selection.reaching().map_or_else(
            || {
                println!("process count: no crate affected");
                Ok(Spawns::default())
            },
            |selection| {
                let execution = Execution::counting()?;
                TestTime::examples(workspace, selection, policy)?;
                TestTime::timings(workspace, selection, policy, &execution)?;
                let counted = execution.processes()?;
                println!("{counted}");
                ledger.spend(Measure::GitProcesses, counted.git().amount());
                Ok(counted)
            },
        )
    }
}
