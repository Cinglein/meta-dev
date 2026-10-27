use std::time::Duration;

use serde::Deserialize;

use super::limit::Limit;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct WatchShare(u64);

impl WatchShare {
    const WHOLE: u64 = 100;

    pub fn bounded(self) -> Option<()> {
        (self.0 > 0 && self.0 <= Self::WHOLE).then_some(())
    }

    pub fn reached(self, spent: Duration, limit: Limit) -> Option<()> {
        let spent = u128::from(Self::WHOLE) * spent.as_millis();
        (spent >= u128::from(self.0) * limit.duration().as_millis()).then_some(())
    }
}
