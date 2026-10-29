mod scope;
mod shard_plan;
mod trigger_event;

pub use scope::Scope;
pub use shard_plan::ShardPlan;
pub use trigger_event::TriggerEvent;

use std::fs::{self, File};

use process_tree::Host;

use scope::MutantsBuild;
use task::AvailableGib;
use task::FailedRuns;
use task::Failure;
use task::Site;
use task::Workspace;
use task::{CommandLine, MutantsArgument};
use task::{MemoryShare, Policy};

#[derive(Debug)]
pub struct Mutants;

impl Mutants {
    const LOCK_FILE: &str = "mutants.lock";
    const PROCESS_NAME: &str = "cargo-mutants";

    pub fn run(workspace: &Workspace, scope: Scope, policy: &Policy) -> Result<(), Failure> {
        let _lock = Self::exclusive_lock(workspace)?;
        Self::refuse_if_another_run_exists()?;
        let budget = Self::refuse_if_memory_is_short(policy)?;
        let build = MutantsBuild::checked(workspace, policy)?;
        let line = build.mutating().argument(MutantsArgument::JobserverTasks).argument(policy.compiler_tasks());
        let runs = scope.applied(line, &build, workspace, policy)?;
        let failed: Vec<CommandLine> = runs
            .iter()
            .map(|run| run.accepted(&build, budget, policy))
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
        fs::create_dir_all(workspace.target().path()).map_err(io)?;
        let lock = File::create(&path).map_err(io)?;
        lock.try_lock().map_err(|_| Failure::LockHeld(path.clone()))?;
        Ok(lock)
    }

    fn refuse_if_another_run_exists() -> Result<(), Failure> {
        if Host::runs(Self::PROCESS_NAME) { Err(Failure::AlreadyRunning(Self::PROCESS_NAME)) } else { Ok(()) }
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
