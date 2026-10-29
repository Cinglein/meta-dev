use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::time::Duration;

use super::super::machine::Machine;
use super::super::pid_list::PidList;
use super::super::process_row::Pid;
use super::super::signal::Signal;
use super::super::start_time::StartTime;
use super::super::target::Target;

const ME: &str = "60";
const LISTED_START: &str = "Mon Oct 26 06:00:00 2026";

#[derive(Debug)]
pub struct FakeMachine<'scene> {
    tables: RefCell<Vec<&'scene str>>,
    cwds: &'scene str,
    uptime: &'scene str,
    acts: RefCell<Vec<String>>,
    starts: BTreeMap<Pid, &'scene str>,
    later_starts: BTreeMap<Pid, &'scene str>,
}

impl<'scene> FakeMachine<'scene> {
    pub fn new(tables: &[&'scene str], cwds: &'scene str, uptime: &'scene str) -> Self {
        let acts = RefCell::default();
        let starts = BTreeMap::new();
        Self { tables: RefCell::new(tables.to_vec()), cwds, uptime, acts, starts, later_starts: BTreeMap::new() }
    }

    pub fn started_after_term(self, pid: &str, start: &'scene str) -> Self {
        let mut later_starts = self.later_starts;
        later_starts.insert(pid.parse().unwrap(), start);
        Self { later_starts, ..self }
    }

    pub fn started_at(self, pid: &str, start: &'scene str) -> Self {
        let mut starts = self.starts;
        starts.insert(pid.parse().unwrap(), start);
        Self { starts, ..self }
    }

    pub fn listed(table: &str) -> String {
        let place = |line: &str| {
            let mut words = line.split_whitespace();
            let (pid, parent, group) = (words.next()?, words.next()?, words.next()?);
            Some(format!("{pid} {parent} {group} {LISTED_START} {}", words.collect::<Vec<_>>().join(" ")))
        };
        table.lines().map(|line| place(line).unwrap_or_else(|| line.to_owned()) + "\n").collect()
    }
}

impl Machine for FakeMachine<'_> {
    fn table(&self) -> Option<String> {
        let mut tables = self.tables.borrow_mut();
        (!tables.is_empty()).then(|| Self::listed(tables.remove(0)))
    }

    fn cwds(&self, pids: &PidList) -> Option<String> {
        self.acts.borrow_mut().push(format!("cwds {pids}"));
        Some(self.cwds.to_owned())
    }

    fn uptime(&self) -> Option<String> {
        Some(self.uptime.to_owned())
    }

    fn started(&self, pid: Pid) -> Option<StartTime> {
        let termed = self.acts.borrow().iter().any(|act| act.starts_with("-TERM"));
        let later = self.later_starts.get(&pid).filter(|_| termed);
        later.or(self.starts.get(&pid)).copied().unwrap_or(LISTED_START).parse().ok()
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
