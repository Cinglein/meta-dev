use super::super::machine::Machine;
use super::super::process_row::Pid;
use super::super::standing::Standing;
use super::super::start_time::StartTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pid: Pid,
    listed: Option<StartTime>,
}

impl Identity {
    pub fn new(pid: Pid, listed: Option<StartTime>) -> Self {
        Self { pid, listed }
    }

    pub fn standing(&self, host: &impl Machine) -> Standing {
        match (host.started(self.pid), &self.listed) {
            (now, listed) if now == *listed => Standing::Same,
            (None, _) => Standing::Gone,
            _ => Standing::Changed,
        }
    }
}
