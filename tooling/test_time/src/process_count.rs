use git_trace::Spawns;
use task::{Failure, Ledger, Measure, Policy, Selection, Workspace};

use super::TestTime;
use super::execution::Execution;

#[derive(Debug)]
pub struct ProcessCount;

impl ProcessCount {
    pub const SWITCH: &'static str = "META_PROCESS_COUNT";

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
                let timings = TestTime::timings(workspace, selection, policy, &execution)?;
                let counted = execution.processes(&timings)?;
                println!("{counted}");
                ledger.spend(Measure::GitProcesses, counted.git().amount());
                Ok(counted)
            },
        )
    }
}
