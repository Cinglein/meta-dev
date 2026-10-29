use std::cell::Cell;
use std::sync::atomic::{AtomicU16, Ordering};

static NEXT: AtomicU16 = AtomicU16::new(0);

thread_local! {
    static LANE: Cell<Option<Lane>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lane(u16);

impl Lane {
    pub fn current() -> Self {
        let lane = LANE.get().unwrap_or_else(|| Self(NEXT.fetch_add(1, Ordering::Relaxed)));
        LANE.set(Some(lane));
        lane
    }

    pub const fn new(word: u16) -> Self {
        Self(word)
    }

    pub fn word(self) -> u16 {
        self.0
    }
}
