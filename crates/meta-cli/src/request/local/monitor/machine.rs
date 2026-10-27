use std::collections::BTreeSet;
use std::time::Duration;

use super::pid_list::PidList;
use super::process_row::Pid;
use super::signal::Signal;
use super::target::Target;

pub trait Machine {
    fn table(&self) -> Option<String>;
    fn cwds(&self, pids: &PidList) -> Option<String>;
    fn uptime(&self) -> Option<String>;
    fn signal(&self, signal: Signal, targets: &BTreeSet<Target>);
    fn pause(&self, grace: Duration);
    fn own_pid(&self) -> Pid;
}
