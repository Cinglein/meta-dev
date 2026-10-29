use std::io;
use std::time::Duration;

use nix::sys::resource::{UsageWho, getrusage};
use nix::sys::time::{TimeVal, TimeValLike};
use task::{Failure, Site};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ChildrenCpu(Duration);

impl ChildrenCpu {
    pub(super) fn now() -> Result<ChildrenCpu, Failure> {
        getrusage(UsageWho::RUSAGE_CHILDREN)
            .map(|usage| Self::summed(usage.user_time(), usage.system_time()))
            .map_err(|errno| Failure::Io {
                site: Site::Workspace,
                error: io::Error::from(errno),
            })
    }

    pub(super) fn since(self, earlier: ChildrenCpu) -> Duration {
        self.0.saturating_sub(earlier.0)
    }

    fn summed(user: TimeVal, system: TimeVal) -> ChildrenCpu {
        ChildrenCpu(Self::spent(user) + Self::spent(system))
    }

    fn spent(time: TimeVal) -> Duration {
        Duration::from_micros(u64::try_from(time.num_microseconds()).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use {super::ChildrenCpu, nix::sys::time::TimeVal, std::process::Command};
    const BUSY: &str = "i=0; while [ $i -lt 20000 ]; do i=$((i+1)); done";
    const NONE: TimeVal = TimeVal::new(0, 0);
    const SPENT: TimeVal = TimeVal::new(0, 2);
    #[test]
    fn a_waited_child_adds_its_user_and_system_cpu_time() {
        let before = ChildrenCpu::now().unwrap();
        Command::new("sh").args(["-c", BUSY]).status().unwrap();
        let spent = ChildrenCpu::now().unwrap().since(before);
        assert!(spent > before.since(before));
        let system = ChildrenCpu::summed(SPENT, SPENT).since(ChildrenCpu::summed(SPENT, NONE));
        assert!(system == ChildrenCpu::spent(SPENT));
    }
}
