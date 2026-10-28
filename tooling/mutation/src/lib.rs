mod scope;
mod shard_plan;
mod trigger_event;

pub use scope::Scope;
pub use shard_plan::ShardPlan;
pub use trigger_event::TriggerEvent;

use meta_door::{Change, Door, Opening};
use std::fs::File;

use sysinfo::{ProcessesToUpdate, System};

use task::AvailableGib;
use task::FailedRuns;
use task::Failure;
use task::Site;
use task::Workspace;
use task::{CargoArgument, CommandLine, MutantsArgument, Program};
use task::{Interval, MemoryShare, Policy};

#[derive(Debug)]
pub struct Mutants;

impl Mutants {
    const LOCK_FILE: &str = "mutants.lock";
    const PROCESS_NAME: &str = "cargo-mutants";

    pub fn run(workspace: &Workspace, scope: Scope, policy: &Policy) -> Result<(), Failure> {
        let _lock = Self::exclusive_lock(workspace)?;
        Self::refuse_if_another_run_exists()?;
        let budget = Self::refuse_if_memory_is_short(policy)?;
        let line = workspace
            .line(Program::Cargo)
            .argument(CargoArgument::Mutants)
            .argument(MutantsArgument::JobserverTasks)
            .argument(policy.compiler_tasks())
            .argument(MutantsArgument::Timeout)
            .argument(policy.interval(Interval::TestTimeout).whole_seconds());
        let runs = scope.applied(line, workspace, policy)?;
        let failed: Vec<CommandLine> = runs
            .iter()
            .map(|run| run.accepted(budget, policy))
            .collect::<Result<Vec<bool>, Failure>>()?
            .into_iter()
            .zip(&runs)
            .filter(|(accepted, _)| !accepted)
            .map(|(_, run)| run.line().clone())
            .collect();
        failed.is_empty().then_some(()).ok_or_else(|| Failure::Runs(FailedRuns::from(failed)))
    }

    fn exclusive_lock(workspace: &Workspace) -> Result<File, Failure> {
        let path = workspace.target().path().join(Self::LOCK_FILE);
        let io = |error| Failure::Io { site: Site::File(path.clone()), error };
        Door::changed(workspace.target().path(), Change::DirAll).map_err(io)?;
        let lock = Door::opened(&path, Opening::Truncate).map_err(io)?;
        lock.try_lock().map_err(|_| Failure::LockHeld(path.clone()))?;
        Ok(lock)
    }

    fn refuse_if_another_run_exists() -> Result<(), Failure> {
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);
        match system.processes_by_name(Self::PROCESS_NAME.as_ref()).next() {
            Some(_) => Err(Failure::AlreadyRunning(Self::PROCESS_NAME)),
            None => Ok(()),
        }
    }

    fn refuse_if_memory_is_short(policy: &Policy) -> Result<AvailableGib, Failure> {
        let available = AvailableGib::measured();
        if available < policy.memory(MemoryShare::Granted) {
            Err(Failure::MemoryShort { available, granted: policy.memory(MemoryShare::Granted) })
        } else {
            Ok(policy.memory(MemoryShare::Granted))
        }
    }
}
