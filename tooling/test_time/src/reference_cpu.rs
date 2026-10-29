use std::fmt;
use std::time::Duration;

use super::children_cpu::ChildrenCpu;
use task::{CommandLine, Failure, Limit, Policy, Program, Stdout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReferenceCpu(Duration);

impl ReferenceCpu {
    const VERSION: &str = "--version";

    pub(super) fn measured(policy: &Policy) -> Result<ReferenceCpu, Failure> {
        let line = CommandLine::new(Program::Git).argument(Self::VERSION);
        let before = ChildrenCpu::now()?;
        (0..policy.limit(Limit::ReferenceSpawns).count())
            .try_for_each(|_| line.captured::<Stdout>().map(|_| ()))?;
        Ok(ReferenceCpu::new(ChildrenCpu::now()?.since(before)))
    }

    pub(super) const fn new(measured: Duration) -> ReferenceCpu {
        ReferenceCpu(measured)
    }
}

impl fmt::Display for ReferenceCpu {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "runner reference {} ms", self.0.as_millis())
    }
}

#[cfg(test)]
mod tests {
    use super::ReferenceCpu;
    use std::time::Duration;
    const MEASURED: Duration = Duration::from_millis(42);
    const SHOWN: &str = "runner reference 42 ms";

    #[test]
    fn a_reference_shows_its_milliseconds() {
        assert_eq!(ReferenceCpu::new(MEASURED).to_string(), SHOWN);
    }
}
