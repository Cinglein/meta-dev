use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct HeadTarget {
    #[serde(default, rename = "committedDate")]
    committed: Option<DateTime<Utc>>,
}

impl HeadTarget {
    pub fn quiet_past(&self, now: DateTime<Utc>, within: TimeDelta) -> Option<()> {
        let due = self.committed?.checked_add_signed(within)?;
        (due < now).then_some(())
    }
}
