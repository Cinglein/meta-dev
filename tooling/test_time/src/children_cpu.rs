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
            .map(|usage| {
                ChildrenCpu(Self::spent(usage.user_time()) + Self::spent(usage.system_time()))
            })
            .map_err(|errno| Failure::Io {
                site: Site::Workspace,
                error: io::Error::from(errno),
            })
    }

    pub(super) fn since(self, earlier: ChildrenCpu) -> Duration {
        self.0.saturating_sub(earlier.0)
    }

    fn spent(time: TimeVal) -> Duration {
        Duration::from_micros(u64::try_from(time.num_microseconds()).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::ChildrenCpu;
    use std::process::Command;
    const BUSY: [&str; 2] = ["-c", "i=0; while [ $i -lt 20000 ]; do i=$((i+1)); done"];

    #[test]
    fn a_waited_child_adds_its_cpu_time() {
        let before = ChildrenCpu::now().unwrap();
        let ran = Command::new("sh").args(BUSY).status().unwrap();
        let spent = ChildrenCpu::now().unwrap().since(before);
        assert!(ran.success() && spent > before.since(before));
    }
}
