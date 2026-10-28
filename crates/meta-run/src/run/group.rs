mod group_id;
mod interrupts;

use std::collections::BTreeSet;
use std::ops::ControlFlow;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::Run;
use crate::env_policy::EnvPolicy;
use crate::ran::Ran;
use crate::run_failure::RunFailure;
use group_id::GroupId;
use interrupts::Interrupts;

static LIVE: Mutex<BTreeSet<GroupId>> = Mutex::new(BTreeSet::new());
static CLOSING: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct Group(GroupId);

impl Group {
    const KILL: &'static str = "kill";
    const STOPPING: [&'static str; 2] = ["-TERM", "-CONT"];
    const KILLING: &'static str = "-KILL";
    const PROBE: &'static str = "-0";
    const END_OF_OPTIONS: &'static str = "--";
    const GRACE: Duration = Duration::from_secs(1);
    const STEP: Duration = Duration::from_millis(5);
    const HANDOFF: Duration = Duration::from_millis(50);
    const SETTLE: Duration = Self::HANDOFF.saturating_add(Self::HANDOFF);
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];

    pub fn accepting() -> Option<()> {
        (!CLOSING.load(Ordering::SeqCst)).then_some(())
    }

    pub fn joined(leader: &mut Child) -> Option<Self> {
        Interrupts::watched();
        let member = Self(GroupId::from(&*leader));
        Self::live().insert(member.0);
        Self::accepting().map(|()| member).or_else(|| {
            Self::ended(leader);
            None
        })
    }

    pub fn ended(leader: &mut Child) {
        let group = GroupId::from(&*leader);
        let begun = Instant::now();
        let deadline = begun + Self::GRACE;
        drop(Self::STOPPING.map(|signal| Self::signalled(&[group], signal, deadline)));
        let _swept = std::iter::repeat(()).try_for_each(|()| Self::swept(group, leader, begun, deadline));
        Run::alone(leader);
    }

    pub fn all_ended() {
        CLOSING.store(true, Ordering::SeqCst);
        let begun = Instant::now();
        let deadline = begun + Self::GRACE;
        let live: Vec<GroupId> = Self::live().iter().copied().collect();
        drop(Self::STOPPING.map(|signal| Self::signalled(&live, signal, deadline)));
        let _closed = std::iter::repeat(()).try_for_each(|()| Self::closed(begun, deadline));
    }

    fn swept(group: GroupId, leader: &mut Child, begun: Instant, deadline: Instant) -> ControlFlow<()> {
        std::thread::sleep(Self::STEP);
        drop(leader.try_wait());
        let late = begun.elapsed().checked_sub(Self::SETTLE);
        drop(late.map(|_| Self::signalled(&[group], Self::KILLING, deadline)));
        let probed = Self::signalled(&[group], Self::PROBE, deadline);
        probed.is_ok_and(|probed| probed.success()).then_some(()).map_or(ControlFlow::Break(()), ControlFlow::Continue)
    }

    fn closed(begun: Instant, deadline: Instant) -> ControlFlow<()> {
        std::thread::sleep(Self::STEP);
        let live: Vec<GroupId> = Self::live().iter().copied().collect();
        let late = begun.elapsed().checked_sub(Self::HANDOFF);
        drop(late.map(|_| Self::signalled(&live, Self::KILLING, deadline)));
        let open = deadline.checked_duration_since(Instant::now()).and(live.first());
        open.map_or(ControlFlow::Break(()), |_| ControlFlow::Continue(()))
    }

    fn signalled(groups: &[GroupId], signal: &str, deadline: Instant) -> Result<Ran, RunFailure> {
        let kill = Run::new(Self::KILL, EnvPolicy::Only(&Self::SEARCH_PATH)).args([signal, Self::END_OF_OPTIONS]);
        groups.iter().fold(kill, |kill, group| group.appended(kill)).helped(deadline)
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
    use crate::run::Run;
    use crate::run_failure::RunFailure;

    const SHELL: &str = "sh";
    const ALONE: [&str; 2] = ["-c", "echo $$ > \"$0\"; exec sleep 5"];
    const LEADER: [&str; 2] = ["-c", "echo $$"];
    const KILL: &str = "kill";
    const ALIVE: [&str; 1] = ["-0"];
    const PATIENT: Duration = Duration::from_secs(5);
    const LIMIT: Duration = Duration::from_millis(60);

    #[test]
    fn a_finished_run_leaves_the_live_groups_and_a_helper_past_its_deadline_is_killed_and_reaped() {
        let recorded = tempfile::NamedTempFile::new().unwrap();
        let helper =
            Run::new(SHELL, EnvPolicy::Inherit).args(ALONE).arg(recorded.path()).helped(Instant::now() + LIMIT);
        let pid = std::fs::read_to_string(recorded.path()).unwrap_or_default();
        let probe = Run::new(KILL, EnvPolicy::Inherit).args(ALIVE).arg(pid.trim()).ran(Instant::now() + PATIENT);
        let leader = Run::new(SHELL, EnvPolicy::Inherit).args(LEADER).stdout(Instant::now() + PATIENT);
        let leader = leader.ok().and_then(|leader| leader.parse::<GroupId>().ok());
        assert!(matches!(helper, Err(RunFailure::TimedOut(_))) && !pid.trim().is_empty());
        assert!(
            probe.is_ok_and(|probe| !probe.success()) && leader.is_some_and(|group| !Group::live().contains(&group))
        );
    }
}
