mod group_id;
mod interrupts;

use std::collections::BTreeSet;
use std::ops::ControlFlow;
use std::process::Child;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::Run;
use crate::env_policy::EnvPolicy;
use crate::environment::Environment;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use group_id::GroupId;
use interrupts::Interrupts;

static LIVE: Mutex<BTreeSet<GroupId>> = Mutex::new(BTreeSet::new());

#[derive(Debug)]
pub struct Group(GroupId);

impl Group {
    const KILL: &'static str = "kill";
    const SIGNALS: [&'static str; 2] = ["-TERM", "-KILL"];
    const PROBE: &'static str = "-0";
    const END_OF_OPTIONS: &'static str = "--";
    const GRACE: Duration = Duration::from_secs(1);
    const STEP: Duration = Duration::from_millis(20);
    const HANDOFF: Duration = Duration::from_millis(100);
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];

    pub fn joined(leader: &Child) -> Self {
        Interrupts::watched();
        let group = GroupId::from(leader);
        Self::live().insert(group);
        Self(group)
    }

    pub fn ended(leader: &mut Child) {
        let group = GroupId::from(&*leader);
        let deadline = Instant::now() + Self::GRACE;
        let signals = std::iter::once(Self::SIGNALS[0]).chain(std::iter::repeat(Self::SIGNALS[1]));
        let _swept = signals.into_iter().try_for_each(|signal| Self::swept(group, leader, signal, deadline));
        Run::alone(leader);
    }

    pub fn all_ended() {
        let live: Vec<GroupId> = Self::live().iter().copied().collect();
        let deadline = Instant::now() + Self::GRACE;
        drop(Self::signalled(&live, Self::SIGNALS[0], deadline));
        std::thread::sleep(Self::HANDOFF);
        drop(Self::signalled(&live, Self::SIGNALS[1], deadline));
    }

    fn swept(group: GroupId, leader: &mut Child, signal: &str, deadline: Instant) -> ControlFlow<()> {
        drop(Self::signalled(&[group], signal, deadline));
        std::thread::sleep(Self::STEP);
        drop(leader.try_wait());
        let probed = Self::signalled(&[group], Self::PROBE, deadline);
        probed.is_ok_and(|probed| probed.success()).then_some(()).map_or(ControlFlow::Break(()), ControlFlow::Continue)
    }

    fn signalled(groups: &[GroupId], signal: &str, deadline: Instant) -> Result<Ran, RunFailure> {
        let kill = Run::ending(Self::KILL, EnvPolicy::Only(&Self::SEARCH_PATH), Environment::current(), Run::alone);
        let kill = groups.iter().fold(kill.args([signal, Self::END_OF_OPTIONS]), |kill, group| group.appended(kill));
        kill.ran(deadline)
    }

    fn live() -> MutexGuard<'static, BTreeSet<GroupId>> {
        LIVE.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        Self::live().remove(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Group, GroupId};
    use crate::env_policy::EnvPolicy;
    use crate::environment::Environment;
    use crate::run::Run;
    use crate::run_failure::RunFailure;

    const SHELL: &str = "sh";
    const ALONE: [&str; 2] = ["-c", "echo $$ > \"$0\"; exec sleep 5"];
    const KILL: &str = "kill";
    const ALIVE: [&str; 1] = ["-0"];
    const PATIENT: Duration = Duration::from_secs(5);
    const LIMIT: Duration = Duration::from_millis(60);

    #[test]
    fn a_helper_past_its_deadline_is_killed_reaped_and_dropped_from_the_live_groups() {
        let recorded = tempfile::NamedTempFile::new().unwrap();
        let helper = Run::ending(SHELL, EnvPolicy::Inherit, Environment::current(), Run::alone).args(ALONE);
        let ran = helper.arg(recorded.path()).ran(Instant::now() + LIMIT);
        let pid = std::fs::read_to_string(recorded.path()).unwrap();
        let probed = Run::new(KILL, EnvPolicy::Inherit).args(ALIVE).arg(pid.trim()).ran(Instant::now() + PATIENT);
        let group: GroupId = pid.parse().unwrap();
        assert!(matches!(ran, Err(RunFailure::TimedOut(_))) && !Group::live().contains(&group));
        assert!(probed.is_ok_and(|probed| !probed.success()));
    }
}
