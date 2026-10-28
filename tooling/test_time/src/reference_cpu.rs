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
    const UNMEASURED: &str = "the reference workload took no cpu time";

    pub(super) fn measured(policy: &Policy) -> Result<ReferenceCpu, Failure> {
        let line = CommandLine::new(Program::Git).argument(Self::VERSION);
        let before = ChildrenCpu::now()?;
        (0..policy.limit(Limit::ReferenceSpawns).count())
            .try_for_each(|_| line.captured::<Stdout>().map(|_| ()))?;
        let measured = ChildrenCpu::now()?.since(before);
        (measured > Duration::ZERO)
            .then_some(ReferenceCpu::new(
                measured,
                policy.interval(Interval::ReferenceCpu).duration(),
            ))
            .ok_or_else(|| Failure::Command(Self::UNMEASURED.to_owned()))
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

#[cfg(test)]
mod tests {
    use super::ReferenceCpu;
    use std::time::Duration;
    const RUNNER: [Duration; 2] = [Duration::from_secs(2), Duration::from_secs(1)];

    #[test]
    fn cpu_on_a_runner_twice_as_slow_as_the_reference_counts_half() {
        let [slow, nominal] = RUNNER;
        assert_eq!(ReferenceCpu::new(slow, nominal).scaled(slow), nominal);
    }
}
