mod killed_process;

pub use killed_process::KilledProcess;

use std::fmt;

#[derive(Debug)]
pub struct KilledProcesses(Vec<KilledProcess>);

impl From<Vec<KilledProcess>> for KilledProcesses {
    fn from(killed: Vec<KilledProcess>) -> KilledProcesses {
        KilledProcesses(killed)
    }
}

impl fmt::Display for KilledProcesses {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        formatter.write_str(&shown.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::{KilledProcess, KilledProcesses};
    use crate::budget::AvailableGib;
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
    const TREE: AvailableGib = AvailableGib::from_bytes(1 << 30);
    const TAIL: &str = "GiB of 1 GiB used";
    const SEPARATOR: &str = ", ";
    #[test]
    fn each_killed_process_is_named_with_its_pid_and_memory_and_the_list_is_comma_separated() {
        let pid = sysinfo::get_current_pid().unwrap();
        let mut system = System::new();
        let memory = ProcessRefreshKind::nothing().with_memory();
        system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, memory);
        let killed = |()| KilledProcess::new(system.process(pid).unwrap(), TREE);
        let shown = KilledProcesses::from(vec![killed(()), killed(())]).to_string();
        let one = shown.split(SEPARATOR).next().unwrap();
        assert!(one.contains(&format!(" (pid {pid}) at ")) && one.ends_with(TAIL));
        assert_eq!(shown, format!("{one}{SEPARATOR}{one}"));
    }
}
