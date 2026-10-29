mod group_id;

use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::Run;
pub(super) use group_id::GroupId;

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
            let left = deadline.checked_duration_since(Instant::now());
            left.and(Self::probed(group, deadline)).map_or(ControlFlow::Break(()), ControlFlow::Continue)
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
        probed.map_or(Some(()), |probed| probed.success().then_some(()))
    }

    pub fn live() -> MutexGuard<'static, BTreeMap<GroupId, Duration>> {
        LIVE.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        let _removed = Self::accepting().and_then(|()| Self::live().remove(&self.0));
    }
}
