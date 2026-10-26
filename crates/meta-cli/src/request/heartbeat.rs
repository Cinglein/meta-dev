mod deadline;
mod gc_hook;
mod heartbeat_flag;
mod heartbeat_record;
mod latest_flags;
mod launchd_plist;
mod snapshot;
mod tick;
mod xml_text;

use std::str::FromStr;

use chrono::Utc;
use meta_pull_query::LiveQuery;

use crate::request::cli_failure::{CliFailure, Misuse, Setup};
use crate::request::live_github::LiveGithub;
use crate::request::local::LocalClone;
use crate::request::output::{Listed, Output};
use crate::request::project::Project;
pub use deadline::Deadline;
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
    Flags,
    Plist,
}

impl FromStr for Heartbeat {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "tick" => Ok(Self::Tick),
            "flags" => Ok(Self::Flags),
            "plist" => Ok(Self::Plist),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown heartbeat command `{other}`"))),
        }
    }
}

impl Heartbeat {
    pub fn answer(self, project: &Project) -> Result<Output, CliFailure> {
        let settings = project.config().heartbeat_settings()?;
        let clone = LocalClone::try_from(project)?;
        match self {
            Self::Tick => {
                let root = project.root().to_path_buf();
                let git_dir = clone.git_dir().to_path_buf();
                let limit = settings.tick_limit();
                Deadline::within(limit, move || {
                    let project = Project::try_from(root.as_path())?;
                    let github = |query: &LiveQuery| LiveGithub::for_project(&project).read(query);
                    Tick::answer(&git_dir, &project.log()?, github, Utc::now())
                })
                .unwrap_or_else(|| Err(CliFailure::raised::<Setup>(format!("tick gave up after {}s", limit.as_secs()))))
            }
            Self::Flags => LatestFlags::answer(clone.git_dir(), &settings, Utc::now()),
            Self::Plist => {
                let program = std::env::current_exe().map_err(CliFailure::raised::<Setup>)?;
                let search_path = std::env::var_os("PATH").unwrap_or_default();
                Ok(Output::Listed(Listed::new(LaunchdPlist::new(program, &clone, &settings, search_path))))
            }
        }
    }
}
