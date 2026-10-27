mod clock_time;
mod command_line;
mod pid;

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use super::culprit::Culprit;
use super::judged_row::JudgedRow;
use super::ownership::Ownership;
use super::target::Target;
use super::unreadable::Unreadable;
use crate::request::local::local_settings::{Cause, CoreShare, MonitorSettings};
use clock_time::ClockTime;
pub use command_line::CommandLine;
pub use pid::Pid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRow {
    pid: Pid,
    parent: Pid,
    group: Pid,
    age: Duration,
    cpu_time: Duration,
    cpu: CoreShare,
    command: CommandLine,
    cwd: Option<PathBuf>,
}

impl ProcessRow {
    const HEADER_LEN: usize = 6;

    pub fn placed(self, cwd: Option<PathBuf>) -> Self {
        Self { cwd, ..self }
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn parent_of(&self, pid: Pid) -> Option<Pid> {
        (self.pid == pid).then_some(self.parent)
    }

    pub fn grouped_with(&self, other: &Self) -> Option<()> {
        (self.group == other.group).then_some(())
    }

    pub fn cpu(&self) -> CoreShare {
        self.cpu
    }

    pub fn cause(&self, settings: &MonitorSettings) -> Option<Cause> {
        settings.cause(self.age, self.cpu_time, (self.parent == Pid::INIT).then_some(()))
    }

    pub fn judged(self, settings: &MonitorSettings, guarded: &[Pid]) -> JudgedRow {
        let spared = guarded.contains(&self.pid).then_some(());
        let spared = spared.or_else(|| settings.spared().iter().find_map(|name| self.command.spares(name)));
        let placed = self.cwd.as_deref().and_then(|cwd| settings.roots().iter().find_map(|root| root.holds(cwd)));
        let named = settings.roots().iter().find_map(|root| self.command.names(root.path()));
        let ownership = match (spared, placed, named) {
            (None, Some(()), _) => Ownership::Agent,
            (None, None, Some(())) => Ownership::Unsure,
            (Some(()), _, _) | (None, None, None) => Ownership::Outside,
        };
        JudgedRow::new(self, ownership)
    }

    pub fn culprit(&self, cause: Cause, whole_group: Option<()>) -> Culprit {
        let target = whole_group.map_or(Target::Process(self.pid), |()| Target::Group(self.group));
        Culprit::new(self.pid, self.command.clone(), cause, target)
    }

    pub fn survives(&self, culprit: &Culprit) -> Option<()> {
        let same_command = (culprit.command() == &self.command).then_some(());
        culprit.target().covers(self.pid, self.group, same_command)
    }
}

impl FromStr for ProcessRow {
    type Err = Unreadable;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (header, command) = words.split_at_checked(Self::HEADER_LEN).ok_or(Unreadable)?;
        let [pid, parent, group, age, cpu_time, cpu] =
            <[&str; Self::HEADER_LEN]>::try_from(header).map_err(|_| Unreadable)?;
        Ok(Self {
            pid: pid.parse()?,
            parent: parent.parse()?,
            group: group.parse()?,
            age: age.parse::<ClockTime>()?.duration(),
            cpu_time: cpu_time.parse::<ClockTime>()?.duration(),
            cpu: cpu.parse::<CoreShare>().map_err(|_| Unreadable)?.hundredth(),
            command: command.join(" ").parse()?,
            cwd: None,
        })
    }
}
