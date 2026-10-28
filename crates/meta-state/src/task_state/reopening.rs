use meta_schema::SchemaVersion;

use crate::stamp::Stamp;
use crate::task_state::progress::Activity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reopening {
    Clearing,
    Keeping,
}

impl Reopening {
    pub fn under(marked: Option<SchemaVersion>) -> Self {
        marked.filter(|version| version.keeps_holders_on_reopen()).map_or(Self::Clearing, |_| Self::Keeping)
    }

    pub fn activity(self, held: &[Stamp]) -> Activity {
        match self {
            Self::Clearing => Activity::unassigned(),
            Self::Keeping => Activity::assigned(held.to_vec()),
        }
    }
}
