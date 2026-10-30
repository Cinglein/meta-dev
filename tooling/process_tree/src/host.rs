use sysinfo::{ProcessesToUpdate, System};

use crate::memory_bytes::MemoryBytes;

#[derive(Debug)]
pub struct Host;

impl Host {
    #[must_use]
    pub fn available() -> MemoryBytes {
        let mut system = System::new();
        system.refresh_memory();
        MemoryBytes::new(system.available_memory())
    }

    #[must_use]
    pub fn runs(name: &str) -> bool {
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);
        system.processes_by_name(name.as_ref()).next().is_some()
    }
}
