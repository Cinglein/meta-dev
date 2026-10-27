mod culprit;
mod finding;
mod judged_row;
mod live_machine;
mod load_average;
mod machine;
mod monitor_trail;
mod ownership;
mod patrol;
mod pid_list;
mod process_row;
mod signal;
mod target;
mod unreadable;

#[cfg(test)]
mod tests;

use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};

use super::LocalClone;
use crate::request::cli_failure::{CliFailure, Misuse};
use crate::request::output::{Listed, Output};
use live_machine::LiveMachine;
use machine::Machine;
use monitor_trail::MonitorTrail;
use patrol::Patrol;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Monitor {
    Tick,
}

impl FromStr for Monitor {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "tick" => Ok(Self::Tick),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown monitor command `{other}`"))),
        }
    }
}

impl Monitor {
    const QUIET: &'static str = "monitor: nothing to stop or flag\n";

    pub fn answer(self, clone: &LocalClone) -> Result<Output, CliFailure> {
        let settings = clone.monitor_settings()?;
        let host = LiveMachine::new(&settings);
        match self {
            Self::Tick => Self::patrolled(clone.git_dir().as_ref(), &Patrol::new(&settings, &host), Utc::now()),
        }
    }

    pub fn recent(clone: &LocalClone, now: DateTime<Utc>) -> Result<Option<Listed>, CliFailure> {
        let entries = MonitorTrail::new(clone.git_dir().as_ref()).since(&clone.monitor_settings()?, now)?;
        Ok((!entries.is_empty()).then(|| entries.iter().fold(Listed::new(""), Listed::then)))
    }

    fn patrolled(git_dir: &Path, patrol: &Patrol<'_, impl Machine>, now: DateTime<Utc>) -> Result<Output, CliFailure> {
        let findings = patrol.findings();
        MonitorTrail::new(git_dir).appended(now, &findings)?;
        Ok(if findings.is_empty() {
            Output::Listed(Listed::new(Self::QUIET))
        } else {
            Output::Flagged(findings.iter().fold(Listed::new(""), Listed::then))
        })
    }
}
