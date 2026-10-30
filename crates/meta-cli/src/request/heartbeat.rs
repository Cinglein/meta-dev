mod gc_hook;
mod heartbeat_record;
mod latest_flags;
mod launchd_plist;
mod pulse;
mod snapshot;
mod tick;

use std::path::PathBuf;
use std::str::FromStr;

use chrono::Utc;
use meta_run::Run;

use crate::request::cli_failure::{CliFailure, Misuse};
use crate::request::local::{LocalClone, Monitor};
use crate::request::output::{Listed, Output};
use crate::request::project::{HeartbeatSettings, Project};
pub use gc_hook::GcHook;
#[cfg(test)]
pub use heartbeat_record::HeartbeatRecord;
pub use latest_flags::LatestFlags;
pub use launchd_plist::LaunchdPlist;
#[cfg(test)]
pub use pulse::Pulse;
#[cfg(test)]
pub use snapshot::Snapshot;
pub use tick::Tick;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heartbeat {
    Tick,
    TickRun,
    GcRun,
    Flags,
    Plist,
}

impl FromStr for Heartbeat {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "tick" => Ok(Self::Tick),
            "tick-run" => Ok(Self::TickRun),
            "gc-run" => Ok(Self::GcRun),
            "flags" => Ok(Self::Flags),
            "plist" => Ok(Self::Plist),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown heartbeat command `{other}`"))),
        }
    }
}

impl Heartbeat {
    const HEARTBEAT: &'static str = "heartbeat";
    const SEARCH_PATH: &'static str = "PATH";
    const TICK_RUN: &'static str = "tick-run";
    const GC_RUN: &'static str = "gc-run";
    const DISABLED: &'static str = "gc disabled";

    pub fn answer(self, project: &Project) -> Result<Output, CliFailure> {
        let settings = HeartbeatSettings::try_from(project.config())?;
        let clone = LocalClone::try_from(project)?;
        match self {
            Self::Tick => {
                Monitor::Tick
                    .answer(project, &clone, &settings)
                    .map_err(|failure| Monitor::noted(&clone, &failure, Utc::now()))
                    .ok();
                let ticked = Tick::supervised(Self::inner(project, Self::TICK_RUN), settings.tick_limit());
                let gc = GcHook::supervised(
                    Self::inner(project, Self::GC_RUN),
                    settings.gc_limit().deadline(),
                    settings.gc_mode(),
                );
                Tick::reported(clone.git_dir().as_ref(), gc)?;
                ticked
            }
            Self::TickRun => Tick::live(project, &clone),
            Self::GcRun => settings.gc_mode().sweep().map_or_else(
                || Ok(Output::Listed(Listed::new(Self::DISABLED))),
                |sweep| clone.collected(project, sweep),
            ),
            Self::Flags => {
                let now = Utc::now();
                let flags = LatestFlags::answer(clone.git_dir().as_ref(), &settings, now)?;
                Ok(flags.raised_with(Monitor::recent(&clone, now)?))
            }
            Self::Plist => Ok(Self::plist(project, &clone, &settings)),
        }
    }

    fn inner(project: &Project, run: &str) -> Run {
        Run::nested(project.environment().program()).args([Self::HEARTBEAT, run]).dir(project.root())
    }

    fn plist(project: &Project, clone: &LocalClone, settings: &HeartbeatSettings) -> Output {
        let environment = project.environment();
        let search_path = environment.variable(Self::SEARCH_PATH).unwrap_or_default().to_owned();
        let program = PathBuf::from(environment.program());
        Output::Listed(Listed::new(LaunchdPlist::new(program, clone, settings, search_path)))
    }
}
