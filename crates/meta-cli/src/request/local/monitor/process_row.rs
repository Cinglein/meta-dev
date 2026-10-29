mod clock_time;
mod command_line;
mod group_id;
mod pid;

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};

use super::command_summary::CommandSummary;
use super::core_share::CoreShare;
use super::culprit::Culprit;
use super::start_time::StartTime;
use super::target::Target;
use super::unreadable::Unreadable;
use super::watch_item::WatchItem;
use crate::request::local::local_settings::{AgentRoot, Cause, MonitorSettings};
use clock_time::ClockTime;
pub use command_line::CommandLine;
pub use group_id::GroupId;
pub use pid::Pid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRow {
    pid: Pid,
    parent: Pid,
    group: GroupId,
    start_time: StartTime,
    age: Duration,
    cpu_time: Duration,
    cpu: CoreShare,
    command: CommandLine,
    cwd: Option<PathBuf>,
}

impl ProcessRow {
    const HEADER_LEN: usize = 11;

    pub fn start_time(&self) -> &StartTime {
        &self.start_time
    }

    pub fn placed(self, cwd: Option<PathBuf>) -> Self {
        Self { cwd, ..self }
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn group(&self) -> GroupId {
        self.group
    }

    pub fn parent_of(&self, pid: Pid) -> Option<Pid> {
        (self.pid == pid).then_some(self.parent)
    }

    pub fn started(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        let age = TimeDelta::from_std(self.age).ok();
        age.and_then(|age| now.checked_sub_signed(age)).unwrap_or(DateTime::<Utc>::MIN_UTC)
    }

    pub fn orphaned(&self) -> Option<()> {
        (self.parent == Pid::INIT).then_some(())
    }

    pub fn wrapper_pid(&self) -> Option<Pid> {
        self.command.wrapper().map(|()| self.pid)
    }

    pub fn spared_pid(&self, settings: &MonitorSettings) -> Option<Pid> {
        settings.spared().iter().find_map(|name| self.command.spares(name)).map(|()| self.pid)
    }

    pub fn root<'settings>(&self, settings: &'settings MonitorSettings) -> Option<&'settings AgentRoot> {
        let cwd = self.cwd.as_deref()?;
        settings.roots().iter().find(|root| root.holds(cwd).is_some())
    }

    pub fn named(&self, settings: &MonitorSettings) -> Option<()> {
        settings.roots().iter().find_map(|root| self.command.names(root.path()))
    }

    pub fn cause(&self, settings: &MonitorSettings) -> Option<Cause> {
        settings.cause(self.age, self.cpu_time, self.orphaned())
    }

    pub fn culprit(&self, cause: Cause, whole_group: Option<()>) -> Culprit {
        let target = whole_group.map_or(Target::Process(self.pid), |()| Target::Group(self.group));
        Culprit::new(self.pid, self.command.clone(), cause, target)
    }

    pub fn command(&self) -> &CommandLine {
        &self.command
    }

    pub fn watched(&self, settings: &MonitorSettings, now: DateTime<Utc>) -> Option<WatchItem> {
        let quiet = self.cause(settings).map_or(Some(()), |_| None);
        quiet.and_then(|()| settings.near(self.age, self.cpu_time))?;
        Some(WatchItem::new(self.pid, CommandSummary::from(&self.command), now, self.cpu_time))
    }
}

impl fmt::Display for ProcessRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let orphaned = self.orphaned().map_or("", |()| " orphaned");
        write!(
            formatter,
            "pid {}{orphaned} age {}s cpu {}s {}: {}",
            self.pid,
            self.age.as_secs(),
            self.cpu_time.as_secs(),
            self.cpu,
            CommandSummary::from(&self.command)
        )
    }
}

impl FromStr for ProcessRow {
    type Err = Unreadable;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (header, command) = words.split_at_checked(Self::HEADER_LEN).ok_or(Unreadable)?;
        let [pid, parent, group, weekday, month, day, clock, year, age, cpu_time, cpu] =
            <[&str; Self::HEADER_LEN]>::try_from(header).map_err(|_| Unreadable)?;
        Ok(Self {
            pid: pid.parse()?,
            parent: parent.parse()?,
            group: group.parse()?,
            start_time: [weekday, month, day, clock, year].join(" ").parse()?,
            age: age.parse::<ClockTime>()?.duration(),
            cpu_time: cpu_time.parse::<ClockTime>()?.duration(),
            cpu: cpu.parse::<CoreShare>().map_err(|_| Unreadable)?.hundredth(),
            command: command.join(" ").parse()?,
            cwd: None,
        })
    }
}
