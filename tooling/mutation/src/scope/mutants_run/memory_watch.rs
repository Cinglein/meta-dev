use std::iter;
use std::process::{Child, ExitStatus};
use std::thread;
use std::time::Instant;

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System};

use task::AvailableGib;
use task::Failure;
use task::KilledProcess;
use task::KilledProcesses;
use task::Site;
use task::{Interval, MemoryShare, Policy};

pub struct MemoryWatch<'policy> {
    budget: AvailableGib,
    policy: &'policy Policy,
    system: System,
    members: Vec<Pid>,
    last_discovery: Instant,
    killed: Vec<KilledProcess>,
}

impl<'policy> MemoryWatch<'policy> {
    pub fn over(child: &Child, budget: AvailableGib, policy: &'policy Policy) -> MemoryWatch<'policy> {
        MemoryWatch {
            budget,
            policy,
            system: System::new(),
            members: vec![Pid::from_u32(child.id())],
            last_discovery: Instant::now(),
            killed: Vec::new(),
        }
    }

    pub fn guard(mut self, child: &mut Child, deadline: Option<Instant>) -> Result<ExitStatus, Failure> {
        let io = |error| Failure::Io { site: Site::File("cargo-mutants".into()), error };
        let every = self.policy.interval(Interval::SampleEvery).duration();
        let polled = iter::from_fn(|| {
            self.sample();
            let late = deadline.filter(|deadline| Instant::now() >= *deadline);
            Some(child.try_wait().map_err(io).map(|waited| (waited, late)))
        })
        .find_map(|waited| match waited {
            Ok((None, None)) => {
                thread::sleep(every);
                None
            }
            Ok((waited, _)) => Some(Ok(waited)),
            Err(error) => Some(Err(error)),
        });
        let status = polled.unwrap_or_else(|| child.wait().map(Some).map_err(io))?.ok_or_else(|| self.ended(child))?;
        if self.killed.is_empty() {
            Ok(status)
        } else {
            Err(Failure::BudgetEnforced { budget: self.budget, killed: KilledProcesses::from(self.killed) })
        }
    }

    pub fn ended(&mut self, child: &mut Child) -> Failure {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let tree: Vec<&Process> =
            self.grown(self.members.clone()).iter().filter_map(|pid| self.system.process(*pid)).collect();
        let used = AvailableGib::from_bytes(tree.iter().map(|process| process.memory()).sum());
        let killed = tree.iter().filter(|process| process.kill()).map(|process| KilledProcess::new(process, used));
        let killed = KilledProcesses::from(killed.collect::<Vec<KilledProcess>>());
        drop(child.wait());
        Failure::PastDeadline { killed }
    }

    fn sample(&mut self) {
        self.system.refresh_memory();
        let members = if self.last_discovery.elapsed() >= self.policy.interval(Interval::DiscoverEvery).duration() {
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            self.last_discovery = Instant::now();
            let alive = self.members.iter().filter(|pid| self.system.process(**pid).is_some()).copied().collect();
            self.grown(alive)
        } else {
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&self.members),
                true,
                ProcessRefreshKind::nothing().with_memory(),
            );
            self.members.clone()
        };
        self.members = members;
        self.killed.extend(self.enforce());
    }

    fn enforce(&self) -> Option<KilledProcess> {
        let tree: Vec<&Process> = self.members.iter().filter_map(|pid| self.system.process(*pid)).collect();
        let used: u64 = tree.iter().map(|process| process.memory()).sum();
        let reserve = self.policy.memory(MemoryShare::Reserve);
        let largest = tree.iter().max_by_key(|process| process.memory())?;
        (used > self.budget.minus(reserve).bytes() || self.system.available_memory() < reserve.bytes()).then(|| {
            largest.kill();
            KilledProcess::new(largest, AvailableGib::from_bytes(used))
        })
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
