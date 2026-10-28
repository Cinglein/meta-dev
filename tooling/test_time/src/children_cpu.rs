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
    use {super::ChildrenCpu, std::process::Command};
    const BUSY: &str = "i=0; while [ $i -lt 20000 ]; do i=$((i+1)); done";
    #[test]
    fn a_waited_child_adds_its_cpu_time_which_a_slower_reference_scales_down() {
        let before = ChildrenCpu::now().unwrap();
        Command::new("sh").args(["-c", BUSY]).status().unwrap();
        let spent = ChildrenCpu::now().unwrap().since(before);
        let slow = crate::reference_cpu::ReferenceCpu::new(spent + spent, spent);
        assert!(spent > before.since(before) && slow.scaled(spent + spent) == spent);
        assert!(slow.to_string().contains("against"));
    }
}
