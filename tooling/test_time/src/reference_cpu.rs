use std::fmt;
use std::time::Duration;

use super::children_cpu::ChildrenCpu;
use task::{CommandLine, Failure, Interval, Limit, Policy, Program, Stdout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReferenceCpu {
    measured: Duration,
    nominal: Duration,
}

impl ReferenceCpu {
    const VERSION: &str = "--version";

    pub(super) fn measured(policy: &Policy) -> Result<ReferenceCpu, Failure> {
        let line = CommandLine::new(Program::Git).argument(Self::VERSION);
        let before = ChildrenCpu::now()?;
        (0..policy.limit(Limit::ReferenceSpawns).count())
            .try_for_each(|_| line.captured::<Stdout>().map(|_| ()))?;
        let measured = ChildrenCpu::now()?.since(before);
        Ok(ReferenceCpu::new(
            measured,
            policy.interval(Interval::ReferenceCpu).duration(),
        ))
    }

    pub(super) const fn new(measured: Duration, nominal: Duration) -> ReferenceCpu {
        ReferenceCpu { measured, nominal }
    }

    pub(super) fn scaled(self, cpu: Duration) -> Duration {
        let nanos = cpu.as_nanos() * self.nominal.as_nanos() / self.measured.as_nanos().max(1);
        Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
    }
}

impl fmt::Display for ReferenceCpu {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "reference {:.2?} against {:.2?}",
            self.measured, self.nominal
        )
    }
}
