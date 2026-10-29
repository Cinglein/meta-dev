use std::ffi::{OsStr, OsString};

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::memory_bytes::MemoryBytes;

#[derive(Debug)]
pub struct Victim {
    name: OsString,
    pid: Pid,
    bytes: MemoryBytes,
}

impl Victim {
    pub(crate) fn of_process(process: &Process) -> Victim {
        Victim { name: process.name().to_os_string(), pid: process.pid(), bytes: MemoryBytes::new(process.memory()) }
    }

    #[must_use]
    pub fn current() -> Option<Victim> {
        let pid = sysinfo::get_current_pid().ok()?;
        let mut system = System::new();
        let memory = ProcessRefreshKind::nothing().with_memory();
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, memory);
        system.process(pid).map(Victim::of_process)
    }

    #[must_use]
    pub fn name(&self) -> &OsStr {
        &self.name
    }

    #[must_use]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    #[must_use]
    pub fn bytes(&self) -> MemoryBytes {
        self.bytes
    }
}
