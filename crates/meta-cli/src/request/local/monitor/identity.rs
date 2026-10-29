use super::machine::Machine;
use super::process_row::Pid;
use super::reused::Reused;
use super::start_time::StartTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pid: Pid,
    started: StartTime,
}

impl Identity {
    pub fn observe(host: &impl Machine, pid: Pid) -> Option<Self> {
        host.started(pid).map(|started| Self { pid, started })
    }

    pub fn confirmed(&self, host: &impl Machine) -> Result<(), Reused> {
        Ok(())
    }
}
