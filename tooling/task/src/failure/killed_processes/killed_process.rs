use std::fmt;

use process_tree::Victim;

use crate::budget::AvailableGib;

#[derive(Debug)]
pub struct KilledProcess {
    victim: Victim,
    tree: AvailableGib,
}

impl KilledProcess {
    #[must_use]
    pub fn new(victim: Victim, tree: AvailableGib) -> KilledProcess {
        KilledProcess { victim, tree }
    }
}

impl fmt::Display for KilledProcess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} (pid {}) at {} of {} used",
            self.victim.name().to_string_lossy(),
            self.victim.pid(),
            AvailableGib::from_bytes(self.victim.bytes().count()),
            self.tree
        )
    }
}
