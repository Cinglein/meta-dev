mod group_id;

use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::Run;
use group_id::GroupId;

static LIVE: Mutex<BTreeMap<GroupId, Duration>> = Mutex::new(BTreeMap::new());
static CLOSING: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct Group(GroupId);

impl Group {
    pub const HANDOFF: Duration = Duration::from_millis(30);
    pub const SETTLE: Duration = Self::HANDOFF.saturating_mul(10);
    const STOPPING: [&'static str; 2] = ["-TERM", "-CONT"];
    const KILLING: &'static str = "-KILL";
    const PROBE: &'static str = "-0";
    const GRACE: Duration = Duration::from_secs(1);
    const STEP: Duration = Duration::from_millis(5);

    pub fn accepting() -> Option<()> {
        (!CLOSING.load(Ordering::SeqCst)).then_some(())
    }

    pub fn joined(leader: &mut Child, patience: Duration) -> Option<Self> {
        let member = Self(GroupId::from(&*leader));
        Self::live().insert(member.0, patience);
        Self::accepting().map(|()| member).or_else(|| {
            Self::ended(leader, patience);
            None
        })
    }

    pub fn ended(leader: &mut Child, patience: Duration) {
        let group = GroupId::from(&*leader);
        let begun = Instant::now();
        let deadline = begun + Self::GRACE;
        drop(Self::STOPPING.map(|signal| GroupId::signalled([&group], signal, deadline)));
        let _swept = std::iter::repeat(()).try_for_each(|()| {
            std::thread::sleep(Self::STEP);
            drop(leader.try_wait());
            drop(begun.elapsed().checked_sub(patience).map(|_| GroupId::signalled([&group], Self::KILLING, deadline)));
            Self::probed(group, deadline).map_or(ControlFlow::Break(()), ControlFlow::Continue)
        });
        Run::alone(leader);
    }

    pub fn all_ended() {
        CLOSING.store(true, Ordering::SeqCst);
        let begun = Instant::now();
        let deadline = begun + Self::GRACE;
        let live = Self::live().clone();
        drop(Self::STOPPING.map(|signal| GroupId::signalled(live.keys(), signal, deadline)));
        let _closed = std::iter::repeat(()).try_for_each(|()| Self::closed(begun, deadline));
    }

    fn closed(begun: Instant, deadline: Instant) -> ControlFlow<()> {
        std::thread::sleep(Self::STEP);
        let live = Self::live().clone();
        let late = live.iter().filter(|(_, patience)| begun.elapsed().checked_sub(**patience).is_some());
        drop(GroupId::signalled(late.map(|(group, _)| group), Self::KILLING, deadline));
        let gone = live.keys().filter(|group| Self::probed(**group, deadline).is_none());
        gone.for_each(|group| {
            Self::live().remove(group);
        });
        let open = Self::live().keys().next().copied();
        deadline
            .checked_duration_since(Instant::now())
            .and(open)
            .map_or(ControlFlow::Break(()), |_| ControlFlow::Continue(()))
    }

    fn probed(group: GroupId, deadline: Instant) -> Option<()> {
        let probed = GroupId::signalled([&group], Self::PROBE, deadline);
        let alive = probed.map_or(Some(()), |probed| probed.success().then_some(()));
        deadline.checked_duration_since(Instant::now()).and(alive)
    }

    fn live() -> MutexGuard<'static, BTreeMap<GroupId, Duration>> {
        LIVE.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        let _removed = Self::accepting().and_then(|()| Self::live().remove(&self.0));
    }
}

#[cfg(test)]
mod tests {
    use super::{Group, GroupId};
    use crate::env_policy::EnvPolicy;
    use crate::run::Run;
    use crate::run_failure::RunFailure;
    use std::ffi::OsStr;
    use std::time::{Duration, Instant};

    const SHELL: &str = "sh";
    const ALONE: [&str; 2] = ["-c", "echo $$ > \"$0\"; exec sleep 5"];
    const LEADER: [&str; 2] = ["-c", "echo $$"];
    const KILL: &str = "kill";
    const ALIVE: [&str; 1] = ["-0"];
    const PATIENT: Duration = Duration::from_secs(5);
    const LIMIT: Duration = Duration::from_millis(60);
    const CLOSING: [&str; 3] = ["--exact", "run::group::tests::closing_runner_refuses_new_spawns", "--ignored"];

    #[test]
    #[ignore = "run in a child process by the test below"]
    fn closing_runner_refuses_new_spawns() {
        Group::all_ended();
        let ran = Run::new(SHELL, EnvPolicy::Inherit).ran(Instant::now() + PATIENT);
        assert!(matches!(ran, Err(RunFailure::Interrupted(_))));
    }

    #[test]
    fn helpers_past_their_deadline_are_reaped_finished_runs_leave_the_live_groups_and_closing_refuses_spawns() {
        let recorded = tempfile::NamedTempFile::new().unwrap();
        let run = |program: &OsStr| Run::new(program, EnvPolicy::Inherit);
        let helped = run(SHELL.as_ref()).args(ALONE).arg(recorded.path()).helped(LIMIT);
        let pid = std::fs::read_to_string(recorded.path()).unwrap_or_default();
        let probe = run(KILL.as_ref()).args(ALIVE).arg(pid.trim()).ran(Instant::now() + PATIENT);
        let leader = run(SHELL.as_ref()).args(LEADER).stdout(Instant::now() + PATIENT);
        let leader = leader.ok().and_then(|leader| leader.parse::<GroupId>().ok());
        let closing = run(std::env::current_exe().unwrap().as_os_str()).args(CLOSING);
        assert!(matches!(helped, Err(RunFailure::TimedOut(_))) && probe.is_ok_and(|probe| !probe.success()));
        assert!(!pid.trim().is_empty() && leader.is_some_and(|group| !Group::live().contains_key(&group)));
        assert!(closing.ran(Instant::now() + PATIENT).is_ok_and(|closing| closing.success()));
    }
}
