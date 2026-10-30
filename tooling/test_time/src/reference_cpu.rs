use std::fmt;
use std::time::Duration;

use rusage::ChildrenCpu;
use task::{CommandLine, Failure, Limit, Policy, Program, Site, Stdout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReferenceCpu(Duration);

impl ReferenceCpu {
    const VERSION: &str = "--version";

    pub(super) fn measured(policy: &Policy) -> Result<ReferenceCpu, Failure> {
        let line = CommandLine::new(Program::Git).argument(Self::VERSION);
        let spent = || {
            ChildrenCpu::now().map_err(|error| Failure::Io {
                site: Site::Workspace,
                error,
            })
        };
        let before = spent()?;
        (0..policy.limit(Limit::ReferenceSpawns).count())
            .try_for_each(|_| line.captured::<Stdout>().map(|_| ()))?;
        Ok(ReferenceCpu::new(spent()?.since(before)))
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
