mod host;
mod memory_bytes;
mod victim;

#[cfg(test)]
mod tests;

use std::process::Child;
use std::time::{Duration, Instant};

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System};

pub use host::Host;
use memory_bytes::MemoryBytes;
pub use victim::Victim;

#[derive(Debug)]
pub struct ProcessTree {
    system: System,
    members: Vec<Pid>,
}

impl ProcessTree {
    #[must_use]
    pub fn rooted_at(child: &Child) -> ProcessTree {
        ProcessTree { system: System::new(), members: vec![Pid::from_u32(child.id())] }
    }

    pub fn sampled(&mut self, discovered: Instant, every: Duration) -> Instant {
        if discovered.elapsed() >= every {
            self.refresh(ProcessesToUpdate::All);
            let alive = self.members.iter().filter(|pid| self.system.process(**pid).is_some()).copied().collect();
            self.members = self.grown(alive);
            Instant::now()
        } else {
            let members = self.members.clone();
            self.refresh(ProcessesToUpdate::Some(&members));
            discovered
        }
    }

    #[must_use]
    pub fn used(&self) -> MemoryBytes {
        self.processes().iter().map(|process| MemoryBytes::new(process.memory())).sum()
    }

    #[must_use]
    pub fn largest_killed(&self) -> Option<Victim> {
        let largest = self.processes().into_iter().max_by_key(|process| process.memory())?;
        largest.kill();
        Some(Victim::of_process(largest))
    }

    #[must_use]
    pub fn all_killed(&self) -> Vec<Victim> {
        self.processes().into_iter().filter(|process| process.kill()).map(Victim::of_process).collect()
    }

    fn processes(&self) -> Vec<&Process> {
        self.members.iter().filter_map(|pid| self.system.process(*pid)).collect()
    }

    fn refresh(&mut self, which: ProcessesToUpdate<'_>) {
        self.system.refresh_processes_specifics(which, true, ProcessRefreshKind::nothing().with_memory());
    }

    fn grown(&self, members: Vec<Pid>) -> Vec<Pid> {
        let children = self.children_outside(&members);
        if children.is_empty() { members } else { self.grown(members.into_iter().chain(children).collect()) }
    }

    fn children_outside(&self, members: &[Pid]) -> Vec<Pid> {
        self.system
            .processes()
            .values()
            .filter(|process| process.thread_kind().is_none())
            .filter(|process| !members.contains(&process.pid()))
            .filter(|process| process.parent().is_some_and(|parent| members.contains(&parent)))
            .map(Process::pid)
            .collect()
    }
}
