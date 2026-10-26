use std::ffi::OsString;
use std::fmt;

use sysinfo::{Pid, Process};

use crate::budget::AvailableGib;

#[derive(Debug)]
pub struct KilledProcess {
    name: OsString,
    pid: Pid,
    own: AvailableGib,
    tree: AvailableGib,
}

impl KilledProcess {
    #[must_use]
    pub fn new(process: &Process, tree: AvailableGib) -> KilledProcess {
        KilledProcess {
            name: process.name().to_os_string(),
            pid: process.pid(),
            own: AvailableGib::from_bytes(process.memory()),
            tree,
        }
    }
}

impl fmt::Display for KilledProcess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} (pid {}) at {} of {} used",
            self.name.to_string_lossy(),
            self.pid,
            self.own,
            self.tree
        )
    }
}
