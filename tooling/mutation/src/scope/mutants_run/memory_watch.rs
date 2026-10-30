use std::iter;
use std::process::{Child, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

use process_tree::{Host, ProcessTree};

use task::AvailableGib;
use task::Failure;
use task::KilledProcess;
use task::KilledProcesses;
use task::Site;
use task::{Interval, MemoryShare, Policy};

pub struct MemoryWatch<'policy> {
    budget: AvailableGib,
    policy: &'policy Policy,
    tree: ProcessTree,
    last_discovery: Instant,
    killed: Vec<KilledProcess>,
}

impl<'policy> MemoryWatch<'policy> {
    pub fn over(child: &Child, budget: AvailableGib, policy: &'policy Policy) -> MemoryWatch<'policy> {
        MemoryWatch {
            budget,
            policy,
            tree: ProcessTree::rooted_at(child),
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
        self.last_discovery = self.tree.sampled(self.last_discovery, Duration::ZERO);
        let used = AvailableGib::from_bytes(self.tree.used().count());
        let killed = self.tree.all_killed().into_iter().map(|victim| KilledProcess::new(victim, used));
        let killed = KilledProcesses::from(killed.collect::<Vec<KilledProcess>>());
        drop(child.wait());
        Failure::PastDeadline { killed }
    }

    fn sample(&mut self) {
        let every = self.policy.interval(Interval::DiscoverEvery).duration();
        self.last_discovery = self.tree.sampled(self.last_discovery, every);
        self.killed.extend(self.enforce());
    }

    fn enforce(&self) -> Option<KilledProcess> {
        let used = self.tree.used().count();
        let reserve = self.policy.memory(MemoryShare::Reserve);
        let victim = (used > self.budget.minus(reserve).bytes() || Host::available().count() < reserve.bytes())
            .then(|| self.tree.largest_killed())
            .flatten()?;
        Some(KilledProcess::new(victim, AvailableGib::from_bytes(used)))
    }
}
