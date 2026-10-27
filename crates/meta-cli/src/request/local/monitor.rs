mod command_summary;
mod core_share;
mod culprit;
mod fault;
mod finding;
mod group_sighting;
mod judged_row;
mod live_machine;
mod load_average;
mod machine;
mod marked_groups;
mod monitor_report;
mod monitor_state;
mod monitor_trail;
mod ownership;
mod patrol;
mod pid_list;
mod process_row;
mod signal;
mod target;
mod unreadable;
mod watch_item;

#[cfg(test)]
mod tests;

use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};

use super::LocalClone;
use crate::request::cli_failure::{CliFailure, Misuse};
use crate::request::local::local_settings::MonitorSettings;
use crate::request::output::{Listed, Output};
use fault::Fault;
use finding::Finding;
use live_machine::LiveMachine;
use machine::Machine;
use monitor_report::MonitorReport;
use monitor_state::MonitorState;
use monitor_trail::MonitorTrail;
use patrol::Patrol;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Monitor {
    Tick,
    Report,
}

impl FromStr for Monitor {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "tick" => Ok(Self::Tick),
            "report" => Ok(Self::Report),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown monitor command `{other}`"))),
        }
    }
}

impl Monitor {
    pub fn answer(self, clone: &LocalClone) -> Result<Output, CliFailure> {
        let git_dir = clone.git_dir().as_ref();
        match self {
            Self::Tick => {
                let settings = clone.monitor_settings()?;
                Self::patrolled(git_dir, &Patrol::new(&settings, &LiveMachine::new(&settings)), Utc::now())
            }
            Self::Report => Ok(Output::Listed(Listed::new(MonitorReport::latest(git_dir)?))),
        }
    }

    pub fn noted(clone: &LocalClone, failure: &CliFailure, at: DateTime<Utc>) {
        let failed = [Finding::Failed(Fault::shown(failure))];
        let settings = clone.monitor_settings().ok().or_else(MonitorSettings::fallback);
        let trail = MonitorTrail::new(clone.git_dir().as_ref());
        settings.map(|settings| trail.appended(at, &failed, &settings).ok());
    }

    pub fn recent(clone: &LocalClone, now: DateTime<Utc>) -> Result<Option<Listed>, CliFailure> {
        match clone.monitor_settings() {
            Ok(settings) => {
                let entries = MonitorTrail::new(clone.git_dir().as_ref()).since(&settings, now)?;
                Ok((!entries.is_empty()).then(|| entries.iter().fold(Listed::new(""), Listed::then)))
            }
            Err(failure) => Ok(Some(Listed::new(Finding::Failed(Fault::shown(failure))))),
        }
    }

    fn patrolled(git_dir: &Path, patrol: &Patrol<'_, impl Machine>, now: DateTime<Utc>) -> Result<Output, CliFailure> {
        let report = patrol.round(&MonitorState::read(git_dir), now);
        report.state().write(git_dir)?;
        report.write(git_dir)?;
        MonitorTrail::new(git_dir).appended(now, report.obvious(), patrol.settings())?;
        Ok(if report.obvious().is_empty() {
            Output::Listed(Listed::new(&report))
        } else {
            Output::Flagged(Listed::new(&report))
        })
    }
}
