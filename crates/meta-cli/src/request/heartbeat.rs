mod gc_hook;
mod heartbeat_record;
mod latest_flags;
mod launchd_plist;
mod snapshot;
mod tick;

use std::process::Command;
use std::str::FromStr;

use chrono::Utc;

use crate::request::cli_failure::{CliFailure, Misuse, Setup};
use crate::request::local::LocalClone;
use crate::request::output::{Listed, Output};
use crate::request::project::{HeartbeatSettings, Project};
#[cfg(test)]
pub use heartbeat_record::HeartbeatRecord;
pub use latest_flags::LatestFlags;
pub use launchd_plist::LaunchdPlist;
#[cfg(test)]
pub use snapshot::Snapshot;
pub use tick::Tick;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heartbeat {
    Tick,
    TickRun,
    Flags,
    Plist,
}

impl FromStr for Heartbeat {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "tick" => Ok(Self::Tick),
            "tick-run" => Ok(Self::TickRun),
            "flags" => Ok(Self::Flags),
            "plist" => Ok(Self::Plist),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown heartbeat command `{other}`"))),
        }
    }
}

impl Heartbeat {
    pub fn answer(self, project: &Project) -> Result<Output, CliFailure> {
        let settings = HeartbeatSettings::try_from(project.config())?;
        let clone = LocalClone::try_from(project)?;
        match self {
            Self::Tick => Tick::supervised(Self::inner_tick(project)?, settings.tick_limit()),
            Self::TickRun => Tick::live(project, &clone),
            Self::Flags => LatestFlags::answer(clone.git_dir().as_ref(), &settings, Utc::now()),
            Self::Plist => Self::plist(&clone, &settings),
        }
    }

    fn inner_tick(project: &Project) -> Result<Command, CliFailure> {
        let mut command = Command::new(std::env::current_exe().map_err(CliFailure::raised::<Setup>)?);
        command.args(["heartbeat", "tick-run"]).current_dir(project.root());
        Ok(command)
    }

    fn plist(clone: &LocalClone, settings: &HeartbeatSettings) -> Result<Output, CliFailure> {
        let program = std::env::current_exe().map_err(CliFailure::raised::<Setup>)?;
        let search_path = std::env::var_os("PATH").unwrap_or_default();
        Ok(Output::Listed(Listed::new(LaunchdPlist::new(program, clone, settings, search_path))))
    }
}
