use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt;
use std::time::Duration;

use super::super::machine::Machine;
use super::super::pid_list::PidList;
use super::super::process_row::Pid;
use super::super::signal::Signal;
use super::super::target::Target;

const ME: &str = "60";

#[derive(Debug)]
pub struct FakeMachine<'scene> {
    tables: RefCell<Vec<&'scene str>>,
    cwds: &'scene str,
    uptime: &'scene str,
    acts: RefCell<Vec<String>>,
}

impl<'scene> FakeMachine<'scene> {
    pub fn new(tables: &[&'scene str], cwds: &'scene str, uptime: &'scene str) -> Self {
        Self { tables: RefCell::new(tables.to_vec()), cwds, uptime, acts: RefCell::default() }
    }
}

impl Machine for FakeMachine<'_> {
    fn table(&self) -> Option<String> {
        let mut tables = self.tables.borrow_mut();
        (!tables.is_empty()).then(|| tables.remove(0).to_owned())
    }

    fn cwds(&self, pids: &PidList) -> Option<String> {
        self.acts.borrow_mut().push(format!("cwds {pids}"));
        Some(self.cwds.to_owned())
    }

    fn uptime(&self) -> Option<String> {
        Some(self.uptime.to_owned())
    }

    fn signal(&self, signal: Signal, targets: &BTreeSet<Target>) {
        let shown: Vec<String> = targets.iter().map(ToString::to_string).collect();
        self.acts.borrow_mut().push(format!("{signal} {}", shown.join(" ")));
    }

    fn pause(&self, grace: Duration) {
        self.acts.borrow_mut().push(format!("pause {}s", grace.as_secs()));
    }

    fn own_pid(&self) -> Pid {
        ME.parse().unwrap()
    }
}

impl fmt::Display for FakeMachine<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.acts.borrow().iter().try_for_each(|act| writeln!(formatter, "{act}"))
    }
}
